#!/usr/bin/env python3
"""Disposable v2->v3 migration rehearsal preparation (no production mutation).

This helper works only on explicitly provided disposable copies/directories.
It can snapshot a v2 data directory, inventory observable state, and emit a
verification manifest for later import/rollback rehearsal when a v3 candidate
exists. Test/support script, not a product CLI.
"""
from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
import random
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def safe_rel(root: Path, path: Path) -> str:
    return str(path.relative_to(root))


def inventory_store(store_dir: Path) -> list[dict[str, Any]]:
    items = []
    if not store_dir.exists():
        return items
    for p in sorted(store_dir.rglob("*")):
        if p.is_file():
            st = p.stat()
            items.append({
                "key": safe_rel(store_dir, p),
                "bytes": st.st_size,
                "sha256": sha256_file(p),
            })
    return items


def inventory_json_dir(dir_path: Path) -> list[dict[str, Any]]:
    items = []
    if not dir_path.exists():
        return items
    for p in sorted(dir_path.glob("*.json")):
        try:
            obj = json.loads(p.read_text(encoding="utf-8"))
        except Exception as e:
            obj = {"_parse_error": str(e)}
        items.append({"file": p.name, "sha256": sha256_file(p), "json": obj})
    return items


def build_manifest(data_dir: Path) -> dict[str, Any]:
    store = inventory_store(data_dir / "store")
    grants = inventory_json_dir(data_dir / "state" / "grants")
    push = inventory_json_dir(data_dir / "state" / "push")
    # include other state files if present (non-recursive safety summary)
    state_extra = []
    state_dir = data_dir / "state"
    if state_dir.exists():
        for p in sorted(state_dir.rglob("*")):
            if p.is_file() and p.suffix != ".json":
                state_extra.append({"path": safe_rel(state_dir, p), "bytes": p.stat().st_size, "sha256": sha256_file(p)})
    manifest = {
        "schema": "skrynia-v2-migration-rehearsal-manifest",
        "version": 1,
        "source_data_dir": str(data_dir),
        "store_objects": store,
        "grants": grants,
        "push_state_files": push,
        "state_extra_files": state_extra,
        "summary": {
            "objects": len(store),
            "object_bytes": sum(i["bytes"] for i in store),
            "grant_files": len(grants),
            "push_files": len(push),
        },
    }
    # deterministic digest of manifest content excluding source_data_dir
    canon = json.dumps({k: v for k, v in manifest.items() if k != "source_data_dir"}, sort_keys=True, separators=(",", ":")).encode()
    manifest["manifest_sha256"] = hashlib.sha256(canon).hexdigest()
    return manifest


def cmd_snapshot(args: argparse.Namespace) -> int:
    src = Path(args.source).resolve()
    dst = Path(args.dest).resolve()
    if not src.exists() or not src.is_dir():
        raise SystemExit(f"source missing/not dir: {src}")
    if dst.exists():
        raise SystemExit(f"dest exists (refusing to overwrite): {dst}")
    # copytree preserves a disposable rehearsal copy
    shutil.copytree(src, dst)
    print(json.dumps({"action": "snapshot", "source": str(src), "dest": str(dst), "ok": True}, sort_keys=True))
    return 0


def cmd_manifest(args: argparse.Namespace) -> int:
    data_dir = Path(args.data_dir).resolve()
    if not data_dir.exists():
        raise SystemExit(f"data dir missing: {data_dir}")
    m = build_manifest(data_dir)
    out = json.dumps(m, indent=2, sort_keys=True)
    if args.output:
        Path(args.output).write_text(out + "\n", encoding="utf-8")
    else:
        print(out)
    print(json.dumps({"action": "manifest", "summary": m["summary"], "manifest_sha256": m["manifest_sha256"]}, sort_keys=True), file=sys.stderr)
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    a = json.loads(Path(args.manifest_a).read_text(encoding="utf-8"))
    b = json.loads(Path(args.manifest_b).read_text(encoding="utf-8"))
    # compare content-relevant fields
    keys = ["store_objects", "grants", "push_state_files", "state_extra_files", "summary"]
    diffs = [k for k in keys if a.get(k) != b.get(k)]
    ok = not diffs and a.get("manifest_sha256") == b.get("manifest_sha256")
    print(json.dumps({"action": "verify", "ok": ok, "diffs": diffs, "a": a.get("manifest_sha256"), "b": b.get("manifest_sha256")}, sort_keys=True))
    return 0 if ok else 1


def _norm_headers(headers: Any) -> dict[str, str]:
    out: dict[str, str] = {}
    for k, v in headers.items():
        out[str(k).lower()] = str(v)
    return out

def do_request(base: str, method: str, path: str, headers: dict[str, str] | None = None, body: bytes | None = None, timeout: float = 15.0):
    url = base.rstrip("/") + path
    req = urllib.request.Request(url, data=body, method=method)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, _norm_headers(resp.headers), resp.read(), None
    except urllib.error.HTTPError as e:
        raw = e.read() if hasattr(e, "read") else b""
        return int(e.code), _norm_headers(getattr(e, "headers", {}) or {}), raw, None
    except Exception as e:  # noqa: BLE001
        return 0, {}, b"", f"{type(e).__name__}: {e}"

def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])

def cmd_live_rehearsal(args: argparse.Namespace) -> int:
    """Executable disposable rehearsal: populate v2, snapshot, manifest, verify rollback copy, restart original."""
    binary = Path(args.v2_binary).resolve()
    if not binary.exists():
        raise SystemExit(f"v2 binary missing: {binary}")
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-migrehearsal-"))
    data_dir = tmp / "v2-data"
    data_dir.mkdir()
    port = free_port()
    env = os.environ.copy()
    env["SKRYNIA_DATA_DIR"] = str(data_dir)
    env["SKRYNIA_HOST"] = "127.0.0.1"
    env["SKRYNIA_PORT"] = str(port)
    env.setdefault("SKRYNIA_URL", f"http://127.0.0.1:{port}")
    base = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen([str(binary)], cwd=str(data_dir), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    result: dict[str, Any] = {"schema": "skrynia-v2-migration-rollback-rehearsal", "ok": False, "steps": []}
    try:
        # wait ready
        deadline = time.time() + 20
        while time.time() < deadline:
            if proc.poll() is not None:
                out = b""
                try:
                    out = proc.stdout.read() if proc.stdout else b""
                except Exception:
                    pass
                raise RuntimeError(f"server exited early rc={proc.returncode} out={out[:300]!r}")
            st, _, _, err = do_request(base, "GET", "/version", timeout=1.0)
            if st == 200 and not err:
                break
            time.sleep(0.1)
        else:
            raise RuntimeError("server not ready")
        # root creds
        cp = subprocess.run([str(binary), "root-credentials"], cwd=str(data_dir), env=env, capture_output=True, text=True, timeout=10)
        if cp.returncode != 0:
            raise RuntimeError(f"root-credentials failed: {cp.stderr[:200]}")
        creds = json.loads(cp.stdout.strip())
        root_id, root_issuer = creds["id"], creds["issuerToken"]
        # issue root user
        st, _, body, err = do_request(base, "POST", f"/grants/{root_id}/users", headers={"Authorization": f"Bearer {root_issuer}"})
        if st != 200 or err:
            raise RuntimeError(f"issue root user failed st={st} err={err} body={body[:120]!r}")
        root_user = json.loads(body.decode())["userToken"]
        # create child grant and representative objects
        grant_body = json.dumps({"locations": "mig/*", "operations": ["create","modify","delete","list","delegate"], "maxBytes": 1_000_000, "maxObjects": 100}).encode()
        st, _, body, err = do_request(base, "POST", "/grants", headers={"Authorization": f"Bearer {root_user}", "Content-Type": "application/json"}, body=grant_body)
        if st != 200:
            raise RuntimeError(f"create grant failed st={st} body={body[:120]!r}")
        child = json.loads(body.decode())
        st, _, body, err = do_request(base, "POST", f"/grants/{child['id']}/users", headers={"Authorization": f"Bearer {child['issuerToken']}"})
        if st != 200:
            raise RuntimeError(f"issue child user failed st={st}")
        child_user = json.loads(body.decode())["userToken"]
        h = {"Authorization": f"Bearer {child_user}"}
        for name, payload in [("a.txt", b"alpha"), ("b.json", b"{\"k\":1}"), ("nested/c.bin", bytes(range(32)))]:
            st, _, bdy, err = do_request(base, "POST", f"/store/mig/{name}", headers=h, body=payload)
            if st != 201:
                raise RuntimeError(f"populate object {name} failed st={st} body={bdy[:120]!r}")
        # stop server before snapshot for filesystem-consistent copy
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill(); proc.wait(timeout=5)
        result["steps"].append({"name": "populate_disposable_v2", "ok": True, "objects": 3, "grants_created": 1})

        src_manifest = build_manifest(data_dir)
        rollback_dir = tmp / "rollback-copy"
        import_dir = tmp / "import-candidate-copy"
        shutil.copytree(data_dir, rollback_dir)
        shutil.copytree(data_dir, import_dir)
        rb_manifest = build_manifest(rollback_dir)
        im_manifest = build_manifest(import_dir)
        ok_copies = src_manifest["manifest_sha256"] == rb_manifest["manifest_sha256"] == im_manifest["manifest_sha256"]
        result["steps"].append({"name": "snapshot_and_manifest_copies", "ok": ok_copies, "source_manifest": src_manifest["manifest_sha256"], "rollback_manifest": rb_manifest["manifest_sha256"], "import_manifest": im_manifest["manifest_sha256"], "summary": src_manifest["summary"]})
        if not ok_copies:
            raise RuntimeError("copy manifests differ")

        # rollback validation: restart original source dir and verify key reads unchanged
        proc = subprocess.Popen([str(binary)], cwd=str(data_dir), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        deadline = time.time() + 20
        while time.time() < deadline:
            st, _, _, err = do_request(base, "GET", "/version", timeout=1.0)
            if st == 200 and not err:
                break
            if proc.poll() is not None:
                raise RuntimeError(f"restart failed rc={proc.returncode}")
            time.sleep(0.1)
        else:
            raise RuntimeError("restart not ready")
        checks = []
        for path, expected in [("/store/mig/a.txt", b"alpha"), ("/store/mig/b.json", b"{\"k\":1}"), ("/store/mig/nested/c.bin", bytes(range(32)))]:
            st, _, body, err = do_request(base, "GET", path)
            checks.append({"path": path, "ok": st == 200 and body == expected and not err, "status": st})
        ok_restart = all(c["ok"] for c in checks)
        result["steps"].append({"name": "restart_original_v2_and_verify_reads", "ok": ok_restart, "checks": checks})
        if not ok_restart:
            raise RuntimeError("post-restart read verification failed")

        # verify rollback copy remained untouched and independently valid (manifest equality as proof of untouched bytes/state files)
        rb_manifest_after = build_manifest(rollback_dir)
        ok_untouched = rb_manifest_after["manifest_sha256"] == src_manifest["manifest_sha256"]
        result["steps"].append({"name": "rollback_copy_untouched", "ok": ok_untouched, "rollback_manifest_after": rb_manifest_after["manifest_sha256"]})
        if not ok_untouched:
            raise RuntimeError("rollback copy changed")

        # v3 import step explicitly blocked/unexecuted but planned
        result["steps"].append({"name": "v3_import_and_post_import_parity", "ok": None, "status": "not_run", "reason": "no integrated v3 kernel+Program0+compat facade candidate available; actual import/export-to-v3 and imported-state differential cannot be executed"})
        result["ok"] = ok_copies and ok_restart and ok_untouched
        result["production_mutation"] = False
        result["disposable_tmp"] = str(tmp)
        result["binary"] = str(binary)
        # keep artifacts for evidence by copying manifests out if requested
        if args.evidence_dir:
            ed = Path(args.evidence_dir)
            ed.mkdir(parents=True, exist_ok=True)
            (ed / "source-manifest.json").write_text(json.dumps(src_manifest, indent=2, sort_keys=True) + "\n")
            (ed / "rollback-manifest.json").write_text(json.dumps(rb_manifest, indent=2, sort_keys=True) + "\n")
            (ed / "import-candidate-manifest.json").write_text(json.dumps(im_manifest, indent=2, sort_keys=True) + "\n")
            (ed / "rehearsal-result.json").write_text(json.dumps(result, indent=2, sort_keys=True, default=str) + "\n")
            result["evidence_dir"] = str(ed)
        print(json.dumps(result, indent=2, sort_keys=True, default=str))
        return 0 if result["ok"] else 1
    finally:
        if proc and proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill(); proc.wait(timeout=5)
        # cleanup temp rehearsal dirs unless evidence requested? keep temp path printed; remove to avoid clutter
        shutil.rmtree(tmp, ignore_errors=True)


def cmd_plan(_: argparse.Namespace) -> int:
    plan = {
        "schema": "skrynia-v3-migration-rehearsal-plan",
        "version": 2,
        "production_mutation": False,
        "steps": [
            "Select pinned v2 source commit/build and disposable source data directory copy (never production).",
            "Populate or capture representative disposable v2 state (objects/grants/push as representable); stop service for filesystem-consistent snapshot.",
            "Snapshot source copy to rollback-copy and candidate-import-copy; record manifests for each and verify equality.",
            "Rollback validation (v2-side): restart original disposable v2 from source copy and verify representative reads/service health; confirm rollback-copy remains untouched.",
            "When v3 kernel+Program0+compat facade candidate is available: import from candidate-import-copy into isolated v3 data dir per approved format/procedure.",
            "Produce post-import observable manifest from v3/facade (objects/metadata/grants/subscriptions/pointers as representable) and diff against source manifest.",
            "Run differential read/write probes on imported disposable instance; record deviations.",
            "Discard v3 candidate data dir; document trigger conditions, exact commands, artifacts, and sign-off evidence.",
        ],
        "executable_now": [
            "live-rehearsal: disposable v2 populate/snapshot/manifest/verify/rollback-copy-untouched/restart-read-verification (no v3 import)",
            "snapshot/manifest/verify subcommands on disposable copies",
        ],
        "cannot_run_now": [
            "Actual import into v3+Program0+facade (candidate not complete/integrated).",
            "Post-import parity verification against v3 candidate.",
            "Production cutover rehearsal on real deployment topology.",
        ],
    }
    print(json.dumps(plan, indent=2, sort_keys=True))
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("snapshot", help="Copy a disposable v2 data dir to a new path")
    p.add_argument("--source", required=True)
    p.add_argument("--dest", required=True)
    p.set_defaults(func=cmd_snapshot)
    p = sub.add_parser("manifest", help="Emit verification manifest for a disposable v2 data dir")
    p.add_argument("--data-dir", required=True)
    p.add_argument("--output")
    p.set_defaults(func=cmd_manifest)
    p = sub.add_parser("verify", help="Compare two manifests for equality")
    p.add_argument("--manifest-a", required=True)
    p.add_argument("--manifest-b", required=True)
    p.set_defaults(func=cmd_verify)
    p = sub.add_parser("plan", help="Print migration/rollback rehearsal plan (no mutation)")
    p.set_defaults(func=cmd_plan)
    p = sub.add_parser("live-rehearsal", help="Run executable disposable v2 populate/snapshot/verify/rollback rehearsal (no v3 import)")
    p.add_argument("--v2-binary", required=True, help="Path to pinned v2 skrynia-server binary")
    p.add_argument("--evidence-dir", help="Directory to write manifests/result artifacts")
    p.set_defaults(func=cmd_live_rehearsal)
    args = ap.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
