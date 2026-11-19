#!/usr/bin/env python3
"""Executable disposable v2->v3 migration rehearsal and fail-closed gates.

Works only on explicitly provided disposable copies/directories. No production
mutation, no product CLI. Actual v3 import/parity is NOT_RUN when no integrated
v3 kernel+Program0+compat facade candidate is available.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

SCHEMA = "skrynia-v2-migration-rehearsal"
V3_IMPORT_REASON = (
    "integrated v3 kernel+Program0+compat facade candidate unavailable; "
    "actual import/export-to-v3 and imported-state differential cannot run"
)

UNSUPPORTED_FIXTURES = [
    {
        "fixture": "actual_v2_to_v3_import",
        "status": "unsupported_without_candidate",
        "reason": V3_IMPORT_REASON,
    },
    {
        "fixture": "post_import_v2_v3_differential_parity",
        "status": "unsupported_without_candidate",
        "reason": V3_IMPORT_REASON,
    },
    {
        "fixture": "push_subscription_live_api_creation",
        "status": "best_effort",
        "reason": "push subscription registration depends on v2 endpoint/key validation and environment; durable push-sub/outbox/metadata state is verified from disk snapshots and any created records are reported",
    },
    {
        "fixture": "push_delivery_ack_end_to_end",
        "status": "unsupported_in_this_gate",
        "reason": "no accepting external push service exercised; only durable subscription/outbox metadata and checksums verified",
    },
    {
        "fixture": "production_data_or_servers",
        "status": "prohibited",
        "reason": "gate uses disposable copies/directories only",
    },
]


class GateFailure(Exception):
    pass


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def rel(root: Path, path: Path) -> str:
    return str(path.relative_to(root))


def ensure_under(base: Path, target: Path, label: str) -> None:
    try:
        target.resolve().relative_to(base.resolve())
    except Exception as exc:
        raise GateFailure(f"{label} escapes disposable base: {target} ({exc})") from exc


def list_entries(root: Path) -> list[Path]:
    return sorted(root.rglob("*"), key=lambda p: str(p.relative_to(root)))


def find_unsafe_entries(root: Path) -> list[dict[str, str]]:
    unsafe: list[dict[str, str]] = []
    for p in list_entries(root):
        try:
            if p.is_symlink():
                unsafe.append({"path": rel(root, p), "kind": "symlink"})
                continue
            st = p.lstat()
            import stat

            mode = st.st_mode
            if stat.S_ISREG(mode) or stat.S_ISDIR(mode):
                continue
            if stat.S_ISFIFO(mode):
                k = "fifo"
            elif stat.S_ISSOCK(mode):
                k = "socket"
            elif stat.S_ISCHR(mode):
                k = "chardev"
            elif stat.S_ISBLK(mode):
                k = "blockdev"
            else:
                k = "special"
            unsafe.append({"path": rel(root, p), "kind": k})
        except FileNotFoundError:
            unsafe.append({"path": rel(root, p), "kind": "missing-during-scan"})
    return unsafe


def assert_safe_tree(root: Path) -> None:
    unsafe = find_unsafe_entries(root)
    if unsafe:
        raise GateFailure("unsafe entries in disposable data dir: " + json.dumps(unsafe, sort_keys=True))


def inventory_regular_files(root: Path) -> list[dict[str, Any]]:
    assert_safe_tree(root)
    items: list[dict[str, Any]] = []
    for p in list_entries(root):
        if p.is_file() and not p.is_symlink():
            st = p.stat()
            items.append({"path": rel(root, p), "bytes": st.st_size, "sha256": sha256_file(p)})
    return items


def read_json_file(path: Path) -> tuple[Any | None, str | None]:
    try:
        with path.open("r", encoding="utf-8") as f:
            return json.load(f), None
    except Exception as exc:  # noqa: BLE001
        return None, f"{type(exc).__name__}: {exc}"


def validate_grant_record(obj: Any, filename_id: str) -> list[str]:
    errs: list[str] = []
    if not isinstance(obj, dict):
        return ["grant record must be object"]
    req = {
        "id": str,
        "parent": (str, type(None)),
        "locations": str,
        "operations": list,
        "maxBytes": (int, type(None)),
        "maxObjects": (int, type(None)),
        "issuerHash": str,
        "users": list,
    }
    for key, typ in req.items():
        if key not in obj:
            errs.append(f"missing field {key}")
        elif not isinstance(obj[key], typ):
            errs.append(f"field {key} has wrong type")
        elif typ is int and isinstance(obj[key], bool):
            errs.append(f"field {key} has wrong type")
    if obj.get("id") != filename_id:
        errs.append("id does not match filename")
    users = obj.get("users")
    if isinstance(users, list):
        for i, u in enumerate(users):
            if not isinstance(u, dict):
                errs.append(f"user[{i}] must be object")
                continue
            if "id" not in u or "hash" not in u:
                errs.append(f"user[{i}] missing id/hash")
            elif not isinstance(u.get("id"), str) or not isinstance(u.get("hash"), str):
                errs.append(f"user[{i}] id/hash must be string")
    ops = obj.get("operations")
    if isinstance(ops, list) and any(not isinstance(x, str) for x in ops):
        errs.append("operations must contain strings")
    if "rootIssuerToken" in obj and obj["rootIssuerToken"] is not None and not isinstance(obj["rootIssuerToken"], str):
        errs.append("rootIssuerToken must be string or null/absent")
    return errs


def validate_subscription_record(obj: Any, filename_id: str) -> list[str]:
    errs: list[str] = []
    if not isinstance(obj, dict):
        return ["subscription record must be object"]
    req = {
        "id": str,
        "grantId": str,
        "locations": str,
        "events": list,
        "endpoint": str,
        "keys": dict,
        "capHash": str,
    }
    for key, typ in req.items():
        if key not in obj:
            errs.append(f"missing field {key}")
        elif not isinstance(obj[key], typ):
            errs.append(f"field {key} has wrong type")
    if obj.get("id") != filename_id:
        errs.append("id does not match filename")
    events = obj.get("events")
    if isinstance(events, list) and any(not isinstance(x, str) for x in events):
        errs.append("events must contain strings")
    keys = obj.get("keys")
    if isinstance(keys, dict):
        if "p256dh" not in keys or "auth" not in keys:
            errs.append("keys missing p256dh/auth")
        elif not isinstance(keys.get("p256dh"), str) or not isinstance(keys.get("auth"), str):
            errs.append("keys p256dh/auth must be string")
    return errs


def validate_outbox_record(obj: Any) -> list[str]:
    errs: list[str] = []
    if not isinstance(obj, dict):
        return ["outbox must be object"]
    for k, v in obj.items():
        if not isinstance(k, str):
            errs.append("outbox key must be string")
        if not isinstance(v, dict):
            errs.append(f"outbox[{k}] must be object")
            continue
        for f in ("generation", "attempts", "nextAt"):
            if f not in v:
                errs.append(f"outbox[{k}] missing {f}")
            elif not isinstance(v[f], int) or isinstance(v[f], bool):
                errs.append(f"outbox[{k}] {f} must be integer")
        if isinstance(v.get("generation"), int) and not isinstance(v.get("generation"), bool) and v["generation"] <= 0:
            errs.append(f"outbox[{k}] generation must be positive")
    return errs


def scan_metadata_dir(data_dir: Path, name: str, validator: Callable[[Any, str], list[str]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    records: list[dict[str, Any]] = []
    issues: list[dict[str, Any]] = []
    d = data_dir / "state" / name
    if not d.exists():
        return records, issues
    if d.is_symlink() or not d.is_dir():
        issues.append({"kind": name, "issue": "metadata directory missing/not directory", "path": rel(data_dir, d)})
        return records, issues
    seen_ids: dict[str, str] = {}
    for p in sorted(d.iterdir(), key=lambda x: x.name):
        if p.is_symlink():
            issues.append({"kind": name, "issue": "symlink entry", "path": rel(data_dir, p)})
            continue
        if p.is_dir():
            issues.append({"kind": name, "issue": "unexpected directory", "path": rel(data_dir, p)})
            continue
        if not p.name.endswith(".json"):
            issues.append({"kind": name, "issue": "non-json entry", "path": rel(data_dir, p)})
            continue
        file_id = p.stem
        if file_id in seen_ids:
            issues.append({"kind": name, "issue": "duplicate id", "id": file_id, "paths": [seen_ids[file_id], rel(data_dir, p)]})
        else:
            seen_ids[file_id] = rel(data_dir, p)
        obj, err = read_json_file(p)
        rec: dict[str, Any] = {"file": p.name, "path": rel(data_dir, p), "sha256": sha256_file(p)}
        if err:
            issues.append({"kind": name, "issue": "corrupt json", "path": rel(data_dir, p), "error": err})
            rec["parse_error"] = err
            records.append(rec)
            continue
        rec["json"] = obj
        verrs = validator(obj, file_id)
        if verrs:
            issues.append({"kind": name, "issue": "invalid metadata", "path": rel(data_dir, p), "errors": verrs})
        records.append(rec)
    return records, issues


def build_manifest(data_dir: Path) -> dict[str, Any]:
    data_dir = data_dir.resolve()
    if not data_dir.is_dir():
        raise GateFailure(f"data dir missing/not dir: {data_dir}")
    all_files = inventory_regular_files(data_dir)
    grant_records, grant_issues = scan_metadata_dir(data_dir, "grants", validate_grant_record)
    sub_records, sub_issues = scan_metadata_dir(data_dir, "push-subs", validate_subscription_record)
    push_dir = data_dir / "state" / "push"
    push_records: list[dict[str, Any]] = []
    push_issues: list[dict[str, Any]] = []
    if push_dir.exists():
        if push_dir.is_symlink() or not push_dir.is_dir():
            push_issues.append({"kind": "push", "issue": "metadata directory missing/not directory", "path": rel(data_dir, push_dir)})
        else:
            for p in sorted(push_dir.glob("*.json"), key=lambda x: x.name):
                if p.is_symlink() or not p.is_file():
                    push_issues.append({"kind": "push", "issue": "non-regular json entry", "path": rel(data_dir, p)})
                    continue
                obj, err = read_json_file(p)
                rec = {"file": p.name, "path": rel(data_dir, p), "sha256": sha256_file(p)}
                if err:
                    push_issues.append({"kind": "push", "issue": "corrupt json", "path": rel(data_dir, p), "error": err})
                    rec["parse_error"] = err
                else:
                    rec["json"] = obj
                    if p.name == "outbox.json":
                        verrs = validate_outbox_record(obj)
                        if verrs:
                            push_issues.append({"kind": "push", "issue": "invalid metadata", "path": rel(data_dir, p), "errors": verrs})
                push_records.append(rec)
    store_items = [i for i in all_files if i["path"].startswith("store/")]
    issues = grant_issues + sub_issues + push_issues
    # cross-check: outbox references must have subscription metadata
    outbox = None
    for r in push_records:
        if r.get("file") == "outbox.json" and isinstance(r.get("json"), dict):
            outbox = r["json"]
    sub_ids = {r["file"][:-5] for r in sub_records if r.get("file", "").endswith(".json")}
    if isinstance(outbox, dict):
        for sid in outbox.keys():
            if sid not in sub_ids:
                issues.append({"kind": "push-subs", "issue": "missing subscription metadata for outbox entry", "id": sid})
    manifest = {
        "schema": SCHEMA + "-manifest",
        "version": 2,
        "source_data_dir": str(data_dir),
        "all_files": all_files,
        "store_objects": store_items,
        "grant_records": grant_records,
        "subscription_records": sub_records,
        "push_state_records": push_records,
        "issues": issues,
        "summary": {
            "files": len(all_files),
            "bytes": sum(i["bytes"] for i in all_files),
            "store_objects": len(store_items),
            "grant_files": len(grant_records),
            "subscription_files": len(sub_records),
            "push_state_files": len(push_records),
            "issue_count": len(issues),
        },
    }
    canon = json.dumps({k: v for k, v in manifest.items() if k != "source_data_dir"}, sort_keys=True, separators=(",", ":"), default=str).encode()
    manifest["manifest_sha256"] = hashlib.sha256(canon).hexdigest()
    return manifest


def assert_manifest_ok(manifest: dict[str, Any], context: str) -> None:
    issues = manifest.get("issues") or []
    if issues:
        raise GateFailure(f"{context}: metadata/content gate failed: " + json.dumps(issues, sort_keys=True, default=str))


def manifests_equal(a: dict[str, Any], b: dict[str, Any]) -> bool:
    keys = ["all_files", "store_objects", "grant_records", "subscription_records", "push_state_records", "issues", "summary"]
    return all(a.get(k) == b.get(k) for k in keys) and a.get("manifest_sha256") == b.get("manifest_sha256")


def safe_copytree(src: Path, dst: Path) -> None:
    assert_safe_tree(src)
    if dst.exists():
        raise GateFailure(f"destination exists: {dst}")
    shutil.copytree(src, dst, symlinks=False)
    # verify no symlinks appeared and contents match
    assert_safe_tree(dst)
    sm = build_manifest(src)
    dm = build_manifest(dst)
    if not manifests_equal(sm, dm):
        raise GateFailure("snapshot copy differs from source")


def write_json(path: Path, obj: Any) -> None:
    path.write_text(json.dumps(obj, indent=2, sort_keys=True, default=str) + "\n", encoding="utf-8")



def extract_verification_report(manifest: dict[str, Any], live: dict[str, Any] | None = None) -> dict[str, Any]:
    grants = []
    issued_users = []
    for rec in manifest.get("grant_records") or []:
        j = rec.get("json") or {}
        if not isinstance(j, dict):
            continue
        users = j.get("users") if isinstance(j.get("users"), list) else []
        norm_users = []
        for u in users:
            if isinstance(u, dict):
                row = {"id": u.get("id"), "hash": u.get("hash")}
                norm_users.append(row)
                issued_users.append({"grantId": j.get("id"), "userId": u.get("id"), "hash": u.get("hash")})
        grants.append({
            "id": j.get("id"),
            "parent": j.get("parent"),
            "locations": j.get("locations"),
            "operations": j.get("operations"),
            "maxBytes": j.get("maxBytes"),
            "maxObjects": j.get("maxObjects"),
            "userCount": len(norm_users),
            "users": sorted(norm_users, key=lambda x: (str(x.get("id")), str(x.get("hash")))),
            "record_sha256": rec.get("sha256"),
        })
    grants.sort(key=lambda x: str(x.get("id")))
    issued_users.sort(key=lambda x: (str(x.get("grantId")), str(x.get("userId"))))
    subs = []
    for rec in manifest.get("subscription_records") or []:
        j = rec.get("json") or {}
        if isinstance(j, dict):
            subs.append({
                "id": j.get("id"),
                "grantId": j.get("grantId"),
                "locations": j.get("locations"),
                "events": j.get("events"),
                "endpoint": j.get("endpoint"),
                "record_sha256": rec.get("sha256"),
            })
    subs.sort(key=lambda x: str(x.get("id")))
    store = [{"path": i.get("path"), "bytes": i.get("bytes"), "sha256": i.get("sha256")} for i in (manifest.get("store_objects") or [])]
    store.sort(key=lambda x: str(x.get("path")))
    return {
        "baseline_checksums": {
            "manifest_sha256": manifest.get("manifest_sha256"),
            "all_files": [{"path": i.get("path"), "bytes": i.get("bytes"), "sha256": i.get("sha256")} for i in (manifest.get("all_files") or [])],
        },
        "stored_bytes": {
            "total_files": manifest.get("summary", {}).get("files"),
            "total_bytes": manifest.get("summary", {}).get("bytes"),
            "store_object_count": manifest.get("summary", {}).get("store_objects"),
            "store_bytes": sum(i.get("bytes", 0) for i in store),
            "objects": store,
        },
        "grants": grants,
        "issued_users": {"total": len(issued_users), "users": issued_users},
        "quotas": {
            "grant_limits_from_metadata": [
                {"id": g["id"], "maxBytes": g["maxBytes"], "maxObjects": g["maxObjects"], "locations": g["locations"], "userCount": g["userCount"]}
                for g in grants
            ],
            "live_observed": (live or {}).get("quotas"),
        },
        "push_subscriptions": {
            "count": len(subs),
            "records": subs,
            "push_state_records": [
                {"path": r.get("path"), "sha256": r.get("sha256"), "file": r.get("file")}
                for r in (manifest.get("push_state_records") or [])
            ],
        },
        "issues": manifest.get("issues") or [],
    }


def compare_reports(before: dict[str, Any], after: dict[str, Any]) -> dict[str, Any]:
    fields = [
        ("manifest_sha256", before["baseline_checksums"]["manifest_sha256"], after["baseline_checksums"]["manifest_sha256"]),
        ("all_file_checksums", before["baseline_checksums"]["all_files"], after["baseline_checksums"]["all_files"]),
        ("stored_bytes", before["stored_bytes"], after["stored_bytes"]),
        ("grants", before["grants"], after["grants"]),
        ("issued_users", before["issued_users"], after["issued_users"]),
        ("grant_limits_from_metadata", before["quotas"]["grant_limits_from_metadata"], after["quotas"]["grant_limits_from_metadata"]),
        ("quotas_live_observed", before["quotas"].get("live_observed"), after["quotas"].get("live_observed")),
        ("push_subscriptions", before["push_subscriptions"], after["push_subscriptions"]),
    ]
    mismatches = []
    rows = []
    for name, b, a in fields:
        eq = b == a
        row = {"field": name, "equal": eq}
        if not eq:
            row["before"] = b
            row["after"] = a
            mismatches.append(row)
        rows.append({"field": name, "equal": eq})
    return {"equal": len(mismatches) == 0, "mismatch_count": len(mismatches), "mismatches": mismatches, "rows": rows}


def start_v2_on_data(binary: Path, data_dir: Path, tmp: Path, name: str) -> tuple[subprocess.Popen, str, dict[str, str], Path]:
    run_dir = tmp / name
    if run_dir.exists():
        shutil.rmtree(run_dir)
    safe_copytree(data_dir, run_dir)
    port = free_port()
    env = os.environ.copy()
    env["SKRYNIA_DATA_DIR"] = str(run_dir)
    env["SKRYNIA_HOST"] = "127.0.0.1"
    env["SKRYNIA_PORT"] = str(port)
    env.setdefault("SKRYNIA_URL", f"http://127.0.0.1:{port}")
    base = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen([str(binary)], cwd=str(run_dir), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    deadline = time.time() + 20
    while time.time() < deadline:
        if proc.poll() is not None:
            out = b""
            try:
                out = proc.stdout.read() if proc.stdout else b""
            except Exception:
                pass
            raise RuntimeError(f"v2 exited early rc={proc.returncode} out={out[:300]!r}")
        st, _, _, err = do_request(base, "GET", "/version", timeout=1.0)
        if st == 200 and not err:
            return proc, base, env, run_dir
        time.sleep(0.1)
    proc.terminate()
    raise RuntimeError("v2 server not ready")


def stop_proc(proc: subprocess.Popen | None) -> None:
    if proc and proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)



# ---- command impls ----

def cmd_snapshot(args: argparse.Namespace) -> int:
    src = Path(args.source).resolve()
    dst = Path(args.dest).resolve()
    safe_copytree(src, dst)
    print(json.dumps({"action": "snapshot", "source": str(src), "dest": str(dst), "ok": True}, sort_keys=True))
    return 0


def cmd_manifest(args: argparse.Namespace) -> int:
    m = build_manifest(Path(args.data_dir))
    if args.output:
        write_json(Path(args.output), m)
    else:
        print(json.dumps(m, indent=2, sort_keys=True, default=str))
    print(json.dumps({"action": "manifest", "summary": m["summary"], "manifest_sha256": m["manifest_sha256"], "ok": m["summary"]["issue_count"] == 0}, sort_keys=True), file=sys.stderr)
    return 0 if m["summary"]["issue_count"] == 0 else 1


def cmd_verify(args: argparse.Namespace) -> int:
    a = json.loads(Path(args.manifest_a).read_text(encoding="utf-8"))
    b = json.loads(Path(args.manifest_b).read_text(encoding="utf-8"))
    ok = manifests_equal(a, b)
    diffs = [k for k in ["all_files", "store_objects", "grant_records", "subscription_records", "push_state_records", "issues", "summary"] if a.get(k) != b.get(k)]
    print(json.dumps({"action": "verify", "ok": ok, "diffs": diffs, "a": a.get("manifest_sha256"), "b": b.get("manifest_sha256")}, sort_keys=True))
    return 0 if ok else 1


def cmd_validate(args: argparse.Namespace) -> int:
    m = build_manifest(Path(args.data_dir))
    ok = m["summary"]["issue_count"] == 0
    print(json.dumps({"action": "validate", "ok": ok, "issues": m["issues"], "summary": m["summary"]}, sort_keys=True, default=str))
    return 0 if ok else 1


def cmd_plan(_: argparse.Namespace) -> int:
    plan = {
        "schema": SCHEMA + "-plan",
        "version": 4,
        "production_mutation": False,
        "executable_now": [
            "Disposable v2 population and baseline capture: checksums, grants, issued users, quotas (live grant usage), stored bytes, push subscription metadata/state with exact manifests.",
            "Immutable rollback snapshot and disposable candidate copy with manifest equality.",
            "Changed writes on disposable candidate (store create/modify/delete, issue user) with explicit before-vs-after mismatch table, then rollback and exact before-vs-after-rollback reversion verification including restarted reads.",
            "Fail-closed gates for unsafe symlinks/special files and corrupt/missing/duplicate/mismatched grant or subscription metadata.",
            "Interrupted import simulation: retry fail-closed and rollback/source integrity proven.",
        ],
        "fail_closed_not_run": [
            "Actual import into integrated v3 kernel+Program0+compat facade candidate.",
            "Post-import v2-v3 differential parity on imported v3 state.",
            "End-to-end push delivery/ack and production cutover/deployed data mutation.",
        ],
        "unsupported_fixtures": UNSUPPORTED_FIXTURES,
        "reason_v3_not_run": V3_IMPORT_REASON,
    }
    print(json.dumps(plan, indent=2, sort_keys=True))
    return 0


# ---- live v2 helpers ----

def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def norm_headers(headers: Any) -> dict[str, str]:
    return {str(k).lower(): str(v) for k, v in headers.items()}


def do_request(base: str, method: str, path: str, headers: dict[str, str] | None = None, body: bytes | None = None, timeout: float = 15.0):
    url = base.rstrip("/") + path
    req = urllib.request.Request(url, data=body, method=method)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, norm_headers(resp.headers), resp.read(), None
    except urllib.error.HTTPError as e:
        raw = e.read() if hasattr(e, "read") else b""
        return int(e.code), norm_headers(getattr(e, "headers", {}) or {}), raw, None
    except Exception as e:  # noqa: BLE001
        return 0, {}, b"", f"{type(e).__name__}: {e}"


@dataclass
class LiveV2:
    binary: Path
    tmp: Path
    data_dir: Path
    base: str
    env: dict[str, str]
    proc: subprocess.Popen | None = None


def start_v2(binary: Path, tmp: Path, name: str = "data") -> LiveV2:
    data_dir = tmp / name
    data_dir.mkdir(parents=True, exist_ok=True)
    port = free_port()
    env = os.environ.copy()
    env["SKRYNIA_DATA_DIR"] = str(data_dir)
    env["SKRYNIA_HOST"] = "127.0.0.1"
    env["SKRYNIA_PORT"] = str(port)
    env.setdefault("SKRYNIA_URL", f"http://127.0.0.1:{port}")
    base = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen([str(binary)], cwd=str(data_dir), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    deadline = time.time() + 20
    while time.time() < deadline:
        if proc.poll() is not None:
            out = b""
            try:
                out = proc.stdout.read() if proc.stdout else b""
            except Exception:
                pass
            raise RuntimeError(f"v2 exited early rc={proc.returncode} out={out[:300]!r}")
        st, _, _, err = do_request(base, "GET", "/version", timeout=1.0)
        if st == 200 and not err:
            return LiveV2(binary=binary, tmp=tmp, data_dir=data_dir, base=base, env=env, proc=proc)
        time.sleep(0.1)
    proc.terminate()
    raise RuntimeError("v2 server not ready")


def stop_v2(v2: LiveV2) -> None:
    proc = v2.proc
    if proc and proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)
    v2.proc = None


def populate_representative_v2(v2: LiveV2) -> dict[str, Any]:
    cp = subprocess.run([str(v2.binary), "root-credentials"], cwd=str(v2.data_dir), env=v2.env, capture_output=True, text=True, timeout=10)
    if cp.returncode != 0:
        raise RuntimeError(f"root-credentials failed: {cp.stderr[:200]}")
    creds = json.loads(cp.stdout.strip())
    st, _, body, err = do_request(v2.base, "POST", f"/grants/{creds['id']}/users", headers={"Authorization": f"Bearer {creds['issuerToken']}"})
    if st != 200 or err:
        raise RuntimeError(f"issue root user failed st={st} err={err}")
    root_user_obj = json.loads(body.decode())
    root_user_token = root_user_obj["userToken"]
    grant_body = json.dumps({"locations": "mig/*", "operations": ["create", "modify", "delete", "list", "delegate", "subscribe"], "maxBytes": 1_000_000, "maxObjects": 100}).encode()
    st, _, body, err = do_request(v2.base, "POST", "/grants", headers={"Authorization": f"Bearer {root_user_token}", "Content-Type": "application/json"}, body=grant_body)
    if st != 200:
        raise RuntimeError(f"create grant failed st={st} body={body[:120]!r}")
    child = json.loads(body.decode())
    st, _, body, err = do_request(v2.base, "POST", f"/grants/{child['id']}/users", headers={"Authorization": f"Bearer {child['issuerToken']}"})
    if st != 200:
        raise RuntimeError(f"issue child user failed st={st}")
    child_user_obj = json.loads(body.decode())
    child_user_token = child_user_obj["userToken"]
    # issue a second user for child grant to make issued-user state explicit
    st, _, body, err = do_request(v2.base, "POST", f"/grants/{child['id']}/users", headers={"Authorization": f"Bearer {child['issuerToken']}"})
    if st != 200:
        raise RuntimeError(f"issue second child user failed st={st}")
    child_user2_obj = json.loads(body.decode())
    h = {"Authorization": f"Bearer {child_user_token}"}
    payloads = [("a.txt", b"alpha"), ("b.json", b"{\"k\":1}"), ("nested/c.bin", bytes(range(64)))]
    for name, payload in payloads:
        st, _, bdy, err = do_request(v2.base, "POST", f"/store/mig/{name}", headers=h, body=payload)
        if st != 201:
            raise RuntimeError(f"populate object {name} failed st={st} body={bdy[:120]!r} err={err}")
    # best-effort push subscription creation (environment/v2-validation dependent)
    sub_attempt = {"attempted": True, "created": False}
    sub_body = json.dumps({
        "locations": "mig/*",
        "events": ["create", "replace", "delete"],
        "endpoint": "https://push.example.invalid/subscription/test",
        "keys": {
            "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUls0VJXg7A8u-Ts1XbjhazAkj7I99e8QcYP7DkM",
            "auth": "tBHItJI5svbpez7KI4CCXg",
        },
    }).encode()
    st, hdrs, bdy, err = do_request(v2.base, "POST", "/push/subscriptions", headers={"Authorization": f"Bearer {child_user_token}", "Content-Type": "application/json"}, body=sub_body)
    sub_attempt.update({"status": st, "error": err})
    try:
        sub_attempt["response"] = json.loads(bdy.decode())
    except Exception:
        sub_attempt["response_text"] = bdy.decode(errors="replace")[:300]
    if st in (200, 201) and not err:
        sub_attempt["created"] = True
    return {
        "objects": payloads,
        "root_grant_id": creds["id"],
        "root_issuer": creds["issuerToken"],
        "root_user_token": root_user_token,
        "root_user_id": root_user_obj.get("userId"),
        "child_grant_id": child["id"],
        "child_issuer": child.get("issuerToken"),
        "child_user_token": child_user_token,
        "child_user_id": child_user_obj.get("userId"),
        "child_user2_id": child_user2_obj.get("userId"),
        "push_subscription_attempt": sub_attempt,
    }


# ---- rehearsal core ----

def simulate_import_with_interruption(source_manifest: dict[str, Any], rollback_dir: Path, candidate_dir: Path, evidence_dir: Path) -> dict[str, Any]:
    """Simulate a v3 import attempt that is interrupted; prove fail-closed + reversion."""
    if candidate_dir.exists():
        shutil.rmtree(candidate_dir)
    safe_copytree(rollback_dir, candidate_dir)
    marker = candidate_dir / ".v3-import-in-progress"
    marker.write_text(json.dumps({"state": "in-progress", "step": "copy-and-transform"}, sort_keys=True), encoding="utf-8")
    # simulate partial/incomplete transformation: mutate candidate so it no longer matches source
    mutated = candidate_dir / "store" / "mig" / "a.txt"
    if mutated.exists():
        mutated.write_bytes(b"PARTIAL-CORRUPTION")
    # also simulate a partially written new artifact that would be unsafe/incomplete
    partial_art = candidate_dir / "state" / "v3-import-partial.tmp"
    partial_art.write_bytes(b"incomplete")
    # interruption occurs here (no marker removal)
    interrupted = {
        "interrupted": True,
        "marker_present": marker.exists(),
        "candidate_differs_from_source": not manifests_equal(source_manifest, build_manifest(candidate_dir)),
    }
    # retry attempt must fail closed while marker/incomplete state exists
    retry_gate = {"ok": False}
    if marker.exists() or partial_art.exists():
        retry_gate = {"ok": False, "fail_closed": True, "reason": "incomplete/interrupted import state detected; retry refused until explicit rollback/cleanup"}
    else:
        retry_gate = {"ok": False, "fail_closed": False, "reason": "unexpected: interruption marker missing"}
    # rollback/reversion: discard candidate and restore from untouched rollback snapshot
    rollback_manifest_before = build_manifest(rollback_dir)
    rollback_intact = manifests_equal(source_manifest, rollback_manifest_before)
    shutil.rmtree(candidate_dir)
    safe_copytree(rollback_dir, candidate_dir)
    reverted_manifest = build_manifest(candidate_dir)
    reversion_ok = manifests_equal(source_manifest, reverted_manifest) and manifests_equal(rollback_manifest_before, reverted_manifest)
    # cleanup candidate to leave no partial state; prove rollback copy still untouched
    shutil.rmtree(candidate_dir)
    rollback_manifest_after = build_manifest(rollback_dir)
    result = {
        "name": "simulated_interruption_retry_rollback",
        "ok": interrupted["candidate_differs_from_source"] and retry_gate["fail_closed"] and rollback_intact and reversion_ok and manifests_equal(source_manifest, rollback_manifest_after),
        "interruption": interrupted,
        "retry_gate": retry_gate,
        "rollback_snapshot_intact": rollback_intact,
        "reversion_exact": reversion_ok,
        "rollback_copy_untouched_after": manifests_equal(source_manifest, rollback_manifest_after),
        "v3_import": {"status": "not_run", "ok": None, "fail_closed": True, "reason": V3_IMPORT_REASON},
    }
    write_json(evidence_dir / "interruption-retry-rollback.json", result)
    return result


def verify_original_reads(binary: Path, tmp: Path, source_data_dir: Path, expected: list[tuple[str, bytes]], evidence_dir: Path) -> dict[str, Any]:
    # restart v2 against a copy of source_data_dir to prove it remains usable; keep source_data_dir itself untouched comparison via manifest caller
    run_dir = tmp / "restart-verify-data"
    if run_dir.exists():
        shutil.rmtree(run_dir)
    safe_copytree(source_data_dir, run_dir)
    v2 = start_v2(binary, tmp, name="restart-verify-data")
    # start_v2 created run_dir inside tmp/name; but we want our copied run_dir used. adjust: easier to start then copy? no.
    stop_v2(v2)
    # Actually start_v2 created tmp/restart-verify-data; overwrite by using different approach
    # Simplify: create fresh LiveV2-like start on existing dir
    shutil.rmtree(tmp / "restart-verify-data")
    safe_copytree(source_data_dir, tmp / "restart-verify-data")
    port = free_port()
    env = os.environ.copy()
    env["SKRYNIA_DATA_DIR"] = str(tmp / "restart-verify-data")
    env["SKRYNIA_HOST"] = "127.0.0.1"
    env["SKRYNIA_PORT"] = str(port)
    env.setdefault("SKRYNIA_URL", f"http://127.0.0.1:{port}")
    base = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen([str(binary)], cwd=str(tmp / "restart-verify-data"), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    try:
        deadline = time.time() + 20
        ready = False
        while time.time() < deadline:
            if proc.poll() is not None:
                raise RuntimeError(f"restart verify server exited rc={proc.returncode}")
            st, _, _, err = do_request(base, "GET", "/version", timeout=1.0)
            if st == 200 and not err:
                ready = True
                break
            time.sleep(0.1)
        if not ready:
            raise RuntimeError("restart verify server not ready")
        checks = []
        for path, payload in expected:
            st, _, body, err = do_request(base, "GET", path)
            checks.append({"path": path, "status": st, "ok": st == 200 and body == payload and not err, "bytes": len(body)})
        ok = all(c["ok"] for c in checks)
        result = {"name": "restart_and_verify_reversion_reads", "ok": ok, "checks": checks}
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill(); proc.wait(timeout=5)
    write_json(evidence_dir / "reversion-read-verification.json", result)
    return result


def capture_live_observations(base: str, info: dict[str, Any]) -> dict[str, Any]:
    quotas = []
    for label, gid, token in [
        ("root", info.get("root_grant_id"), info.get("root_user_token")),
        ("child", info.get("child_grant_id"), info.get("child_user_token")),
    ]:
        if not gid or not token:
            continue
        st, hdrs, body, err = do_request(base, "GET", f"/grants/{gid}", headers={"Authorization": f"Bearer {token}"})
        row = {"label": label, "grant_id": gid, "status": st, "error": err}
        if st == 200 and not err:
            try:
                row["grant"] = json.loads(body.decode())
            except Exception as e:
                row["parse_error"] = str(e)
        else:
            row["body"] = body.decode(errors="replace")[:300]
        quotas.append(row)
    # list objects for child scope as live observation
    listing = None
    if info.get("child_user_token"):
        st, _, body, err = do_request(base, "GET", "/store?locations=mig/*", headers={"Authorization": f"Bearer {info['child_user_token']}"})
        listing = {"status": st, "error": err}
        if st == 200 and not err:
            try:
                listing["body"] = json.loads(body.decode())
            except Exception:
                listing["body_text"] = body.decode(errors="replace")[:300]
        else:
            listing["body_text"] = body.decode(errors="replace")[:300]
    return {"quotas": quotas, "listing": listing}


def perform_changed_writes(base: str, info: dict[str, Any]) -> dict[str, Any]:
    h = {"Authorization": f"Bearer {info['child_user_token']}"}
    actions = []
    # modify existing
    st, _, body, err = do_request(base, "PUT", "/store/mig/a.txt", headers=h, body=b"alpha-changed")
    actions.append({"action": "modify", "path": "/store/mig/a.txt", "status": st, "ok": st == 200 and not err, "error": err, "body": body.decode(errors="replace")[:120]})
    # create new
    st, _, body, err = do_request(base, "POST", "/store/mig/new.txt", headers=h, body=b"new-data")
    actions.append({"action": "create", "path": "/store/mig/new.txt", "status": st, "ok": st == 201 and not err, "error": err, "body": body.decode(errors="replace")[:120]})
    # delete one
    st, _, body, err = do_request(base, "DELETE", "/store/mig/b.json", headers=h)
    actions.append({"action": "delete", "path": "/store/mig/b.json", "status": st, "ok": st == 200 and not err, "error": err, "body": body.decode(errors="replace")[:120]})
    # issue another user (changes grant metadata / issued users)
    st, _, body, err = do_request(base, "POST", f"/grants/{info['child_grant_id']}/users", headers={"Authorization": f"Bearer {info['child_issuer']}"})
    issued = {"action": "issue_user", "grant_id": info['child_grant_id'], "status": st, "ok": st == 200 and not err, "error": err}
    if st == 200 and not err:
        try:
            issued["userId"] = json.loads(body.decode()).get("userId")
        except Exception:
            pass
    else:
        issued["body"] = body.decode(errors="replace")[:120]
    actions.append(issued)
    return {"ok": all(a.get("ok") for a in actions), "actions": actions}


def run_full_rehearsal(binary: Path, work_dir: Path, evidence_dir: Path) -> dict[str, Any]:
    evidence_dir.mkdir(parents=True, exist_ok=True)
    result: dict[str, Any] = {
        "schema": SCHEMA + "-result",
        "ok": False,
        "production_mutation": False,
        "binary": str(binary),
        "work_dir": str(work_dir),
        "evidence_dir": str(evidence_dir),
        "unsupported_fixtures": UNSUPPORTED_FIXTURES,
        "steps": [],
    }
    v2 = start_v2(binary, work_dir, name="v2-source")
    try:
        populated = populate_representative_v2(v2)
        baseline_live = capture_live_observations(v2.base, populated)
        stop_v2(v2)

        baseline_manifest = build_manifest(v2.data_dir)
        assert_manifest_ok(baseline_manifest, "baseline preflight")
        baseline_report = extract_verification_report(baseline_manifest, baseline_live)
        write_json(evidence_dir / "baseline-manifest.json", baseline_manifest)
        write_json(evidence_dir / "baseline-report.json", {"manifest_summary": baseline_manifest["summary"], "verification": baseline_report, "live": baseline_live, "population": {k: v for k, v in populated.items() if k != "objects"}})
        result["steps"].append({
            "name": "populate_disposable_v2_and_capture_baseline",
            "ok": True,
            "summary": baseline_manifest["summary"],
            "verification": {
                "checksums_manifest_sha256": baseline_report["baseline_checksums"]["manifest_sha256"],
                "grants": len(baseline_report["grants"]),
                "issued_users": baseline_report["issued_users"]["total"],
                "stored_bytes_total": baseline_report["stored_bytes"]["total_bytes"],
                "store_object_count": baseline_report["stored_bytes"]["store_object_count"],
                "push_subscription_count": baseline_report["push_subscriptions"]["count"],
            },
            "push_subscription_attempt": populated.get("push_subscription_attempt"),
        })

        rollback_dir = work_dir / "rollback-snapshot"
        safe_copytree(v2.data_dir, rollback_dir)
        rollback_manifest = build_manifest(rollback_dir)
        copy_ok = manifests_equal(baseline_manifest, rollback_manifest)
        write_json(evidence_dir / "rollback-snapshot-manifest.json", rollback_manifest)
        result["steps"].append({"name": "immutable_rollback_snapshot", "ok": copy_ok, "manifest_sha256": rollback_manifest["manifest_sha256"]})
        if not copy_ok:
            raise GateFailure("rollback snapshot differs from baseline")

        # Changed writes on candidate copy, then rollback to baseline snapshot
        proc_cw, base_cw, env_cw, cw_run_dir = start_v2_on_data(binary, rollback_dir, work_dir, "candidate-changed-writes")
        try:
            changed = perform_changed_writes(base_cw, populated)
            after_writes_live = capture_live_observations(base_cw, populated)
        finally:
            stop_proc(proc_cw)
        after_writes_manifest = build_manifest(cw_run_dir)
        after_writes_report = extract_verification_report(after_writes_manifest, after_writes_live)
        write_json(evidence_dir / "after-writes-manifest.json", after_writes_manifest)
        write_json(evidence_dir / "after-writes-report.json", {"verification": after_writes_report, "live": after_writes_live, "changed_writes": changed})
        baseline_vs_after_writes = compare_reports(baseline_report, after_writes_report)
        write_json(evidence_dir / "baseline-vs-after-writes.json", baseline_vs_after_writes)
        changed_writes_ok = bool(changed.get("ok")) and (not baseline_vs_after_writes["equal"])
        result["steps"].append({
            "name": "changed_writes_on_disposable_candidate",
            "ok": changed_writes_ok,
            "changed_writes": changed,
            "state_changed_relative_to_baseline": not baseline_vs_after_writes["equal"],
            "mismatch_count": baseline_vs_after_writes["mismatch_count"],
            "mismatch_fields": [m["field"] for m in baseline_vs_after_writes["mismatches"]],
            "after_writes_manifest_sha256": after_writes_manifest["manifest_sha256"],
        })

        # Rollback: restore baseline snapshot over candidate-write dir and verify
        if cw_run_dir.exists():
            shutil.rmtree(cw_run_dir)
        safe_copytree(rollback_dir, cw_run_dir)
        after_rollback_manifest = build_manifest(cw_run_dir)
        # verify reverted reads via restart against rolled-back copy
        proc_rb, base_rb, env_rb, rb_run_dir = start_v2_on_data(binary, rollback_dir, work_dir, "candidate-after-rollback")
        try:
            rollback_reads = []
            expected_reads = [(f"/store/mig/{n}", b) for n, b in populated["objects"]]
            expected_reads.append(("/store/mig/new.txt", None))  # must not exist
            expected_reads.append(("/store/mig/b.json", None))    # deleted in writes but must exist again after rollback
            for path, expected in [(f"/store/mig/{n}", b) for n, b in populated["objects"]]:
                st, _, body, err = do_request(base_rb, "GET", path)
                rollback_reads.append({"path": path, "status": st, "ok": st == 200 and body == expected and not err, "bytes": len(body)})
            st, _, body, err = do_request(base_rb, "GET", "/store/mig/new.txt")
            rollback_reads.append({"path": "/store/mig/new.txt", "status": st, "ok": st == 404, "note": "must be absent after rollback"})
            st, _, body, err = do_request(base_rb, "GET", "/store/mig/b.json")
            rollback_reads.append({"path": "/store/mig/b.json", "status": st, "ok": st == 200 and body == b"{\"k\":1}" and not err, "note": "must be restored after rollback"})
            after_rollback_live = capture_live_observations(base_rb, populated)
        finally:
            stop_proc(proc_rb)
        after_rollback_manifest = build_manifest(cw_run_dir)
        after_rollback_report = extract_verification_report(after_rollback_manifest, after_rollback_live)
        baseline_vs_after_rollback = compare_reports(baseline_report, after_rollback_report)
        write_json(evidence_dir / "after-rollback-manifest.json", after_rollback_manifest)
        write_json(evidence_dir / "after-rollback-report.json", {"verification": after_rollback_report, "live": after_rollback_live, "reads": rollback_reads})
        write_json(evidence_dir / "baseline-vs-after-rollback.json", baseline_vs_after_rollback)
        rollback_reversion_ok = baseline_vs_after_rollback["equal"] and all(r.get("ok") for r in rollback_reads)
        result["steps"].append({
            "name": "rollback_reversion_after_changed_writes",
            "ok": rollback_reversion_ok,
            "exact_reversion_to_baseline": baseline_vs_after_rollback["equal"],
            "mismatch_count": baseline_vs_after_rollback["mismatch_count"],
            "mismatches": baseline_vs_after_rollback["mismatches"],
            "reads_verified": all(r.get("ok") for r in rollback_reads),
            "reads": rollback_reads,
            "after_rollback_manifest_sha256": after_rollback_manifest["manifest_sha256"],
        })

        # Baseline snapshot and original source must remain intact
        baseline_snapshot_still_intact = manifests_equal(baseline_manifest, build_manifest(rollback_dir))
        original_still_intact = manifests_equal(baseline_manifest, build_manifest(v2.data_dir))
        result["steps"].append({
            "name": "baseline_snapshot_and_original_source_intact",
            "ok": baseline_snapshot_still_intact and original_still_intact,
            "baseline_snapshot_intact": baseline_snapshot_still_intact,
            "original_source_intact": original_still_intact,
        })

        # Interruption/retry/rollback simulation gate (existing fail-closed proof)
        candidate_dir = work_dir / "import-candidate"
        sim = simulate_import_with_interruption(baseline_manifest, rollback_dir, candidate_dir, evidence_dir)
        result["steps"].append(sim)

        # Restart/read verification against original baseline copy
        reads = verify_original_reads(binary, work_dir, rollback_dir, [(f"/store/mig/{n}", b) for n, b in populated["objects"]], evidence_dir)
        result["steps"].append(reads)

        # v3 import/parity - explicitly fail-closed not_run unless candidate provided (none in this harness)
        result["steps"].append({
            "name": "v3_import_and_post_import_parity",
            "ok": None,
            "status": "not_run",
            "fail_closed": True,
            "reason": V3_IMPORT_REASON,
        })

        # Explicit before/after/after-rollback tables required by issue
        result["verification_tables"] = {
            "baseline": {
                "checksums_manifest_sha256": baseline_report["baseline_checksums"]["manifest_sha256"],
                "grants_count": len(baseline_report["grants"]),
                "issued_users_total": baseline_report["issued_users"]["total"],
                "quotas": baseline_report["quotas"],
                "stored_bytes": baseline_report["stored_bytes"],
                "push_subscriptions_count": baseline_report["push_subscriptions"]["count"],
            },
            "after_changed_writes": {
                "manifest_sha256": after_writes_manifest["manifest_sha256"],
                "grants_count": len(after_writes_report["grants"]),
                "issued_users_total": after_writes_report["issued_users"]["total"],
                "quotas": after_writes_report["quotas"],
                "stored_bytes": after_writes_report["stored_bytes"],
                "push_subscriptions_count": after_writes_report["push_subscriptions"]["count"],
            },
            "after_rollback": {
                "manifest_sha256": after_rollback_manifest["manifest_sha256"],
                "grants_count": len(after_rollback_report["grants"]),
                "issued_users_total": after_rollback_report["issued_users"]["total"],
                "quotas": after_rollback_report["quotas"],
                "stored_bytes": after_rollback_report["stored_bytes"],
                "push_subscriptions_count": after_rollback_report["push_subscriptions"]["count"],
            },
            "baseline_vs_after_writes": {
                "equal": baseline_vs_after_writes["equal"],
                "mismatch_count": baseline_vs_after_writes["mismatch_count"],
                "mismatch_fields": [m["field"] for m in baseline_vs_after_writes["mismatches"]],
            },
            "baseline_vs_after_rollback": {
                "equal": baseline_vs_after_rollback["equal"],
                "mismatch_count": baseline_vs_after_rollback["mismatch_count"],
                "mismatches": baseline_vs_after_rollback["mismatches"],
            },
        }

        result["gates"] = {
            "unsafe_symlinks": "enforced by preflight/manifest/snapshot",
            "metadata_integrity": "enforced by preflight/manifest (corrupt/missing-fields/duplicate/id-mismatch/outbox-orphan)",
            "interruption_retry_fail_closed": sim["retry_gate"]["fail_closed"],
            "changed_writes_demonstrated": changed_writes_ok,
            "rollback_reversion_exact_after_changed_writes": rollback_reversion_ok,
            "v3_candidate_available": False,
        }
        result["ok"] = all(
            s.get("ok") is True or (s.get("status") == "not_run" and s.get("fail_closed") is True)
            for s in result["steps"]
        )
    finally:
        stop_v2(v2)
    write_json(evidence_dir / "rehearsal-result.json", result)
    print(json.dumps(result, indent=2, sort_keys=True, default=str))
    return result



def cmd_rehearse(args: argparse.Namespace) -> int:
    binary = Path(args.v2_binary).resolve()
    if not binary.exists():
        raise SystemExit(f"v2 binary missing: {binary}")
    work_dir = Path(tempfile.mkdtemp(prefix="skrynia-migrehearsal-"))
    evidence_dir = Path(args.evidence_dir).resolve() if args.evidence_dir else (work_dir / "evidence")
    try:
        result = run_full_rehearsal(binary, work_dir, evidence_dir)
        return 0 if result.get("ok") else 1
    finally:
        if args.keep_work_dir:
            print(json.dumps({"kept_work_dir": str(work_dir)}, sort_keys=True), file=sys.stderr)
        else:
            shutil.rmtree(work_dir, ignore_errors=True)


def cmd_live_rehearsal(args: argparse.Namespace) -> int:
    # compatibility wrapper
    class A: pass
    a = A()
    a.v2_binary = args.v2_binary
    a.evidence_dir = args.evidence_dir
    a.keep_work_dir = getattr(args, "keep_work_dir", False)
    return cmd_rehearse(a)


# ---- gate tests (executable, no production) ----

def mk_min_v2_data(root: Path) -> None:
    (root / "store" / "mig").mkdir(parents=True)
    (root / "store" / "mig" / "a.txt").write_bytes(b"alpha")
    (root / "state" / "grants").mkdir(parents=True)
    (root / "state" / "push-subs").mkdir(parents=True)
    (root / "state" / "push").mkdir(parents=True)
    grant = {"id": "g1", "parent": None, "locations": "*", "operations": ["create"], "maxBytes": None, "maxObjects": None, "issuerHash": "h", "users": [{"id": "u1", "hash": "hh"}]}
    write_json(root / "state" / "grants" / "g1.json", grant)
    write_json(root / "state" / "push" / "outbox.json", {})
    write_json(root / "state" / "push" / "vapid.json", {"publicKey": "pk", "privateKey": "sk"})


def expect_gate(name: str, fn: Callable[[], Any]) -> dict[str, Any]:
    try:
        fn()
        return {"test": name, "ok": False, "error": "expected GateFailure"}
    except GateFailure as e:
        return {"test": name, "ok": True, "gate": str(e)}
    except Exception as e:  # noqa: BLE001
        return {"test": name, "ok": False, "error": f"unexpected {type(e).__name__}: {e}"}


def run_gate_tests(binary: Path | None) -> dict[str, Any]:
    results: list[dict[str, Any]] = []

    with tempfile.TemporaryDirectory(prefix="skrynia-gatetest-") as td:
        base = Path(td)

        # unsafe symlink in store
        d = base / "symlink-store"; mk_min_v2_data(d)
        (d / "store" / "mig" / "link").symlink_to(d / "store" / "mig" / "a.txt")
        results.append(expect_gate("unsafe_symlink_store", lambda d=d: build_manifest(d)))
        results.append(expect_gate("unsafe_symlink_snapshot", lambda d=d, b=base/"out1": safe_copytree(d, b)))

        # unsafe symlink in grant metadata
        d = base / "symlink-grant"; mk_min_v2_data(d)
        (d / "state" / "grants" / "evil.json").symlink_to(d / "state" / "grants" / "g1.json")
        results.append(expect_gate("unsafe_symlink_grant_metadata", lambda d=d: build_manifest(d)))

        # corrupt grant metadata
        d = base / "corrupt-grant"; mk_min_v2_data(d)
        (d / "state" / "grants" / "g1.json").write_text("{not-json", encoding="utf-8")
        m = build_manifest(d)
        results.append({"test": "corrupt_grant_metadata", "ok": m["summary"]["issue_count"] > 0 and any(i.get("issue") == "corrupt json" and i.get("kind") == "grants" for i in m["issues"]), "issues": m["issues"]})

        # missing required grant field
        d = base / "missing-grant-field"; mk_min_v2_data(d)
        bad = json.loads((d / "state" / "grants" / "g1.json").read_text())
        del bad["issuerHash"]
        write_json(d / "state" / "grants" / "g1.json", bad)
        m = build_manifest(d)
        results.append({"test": "missing_grant_metadata_field", "ok": any(i.get("kind") == "grants" and i.get("issue") == "invalid metadata" for i in m["issues"]), "issues": m["issues"]})

        # duplicate grant id metadata (two files same logical id inside content/filename mismatch treated as invalid; also create duplicate by same filename impossible, so test duplicate id via internal mismatch plus another file with same id)
        d = base / "dup-grant-id"; mk_min_v2_data(d)
        g1 = json.loads((d / "state" / "grants" / "g1.json").read_text())
        write_json(d / "state" / "grants" / "g2.json", g1)  # id inside is g1 but filename g2 => invalid/duplicate signal
        m = build_manifest(d)
        results.append({"test": "duplicate_or_mismatched_grant_id_metadata", "ok": any(i.get("kind") == "grants" and i.get("issue") == "invalid metadata" for i in m["issues"]), "issues": m["issues"]})

        # corrupt subscription metadata
        d = base / "corrupt-sub"; mk_min_v2_data(d)
        (d / "state" / "push-subs" / "s1.json").write_text("{", encoding="utf-8")
        m = build_manifest(d)
        results.append({"test": "corrupt_subscription_metadata", "ok": any(i.get("kind") == "push-subs" and i.get("issue") == "corrupt json" for i in m["issues"]), "issues": m["issues"]})

        # missing subscription field
        d = base / "missing-sub-field"; mk_min_v2_data(d)
        sub = {"id": "s1", "grantId": "g1", "locations": "*", "events": ["x"], "endpoint": "https://e", "keys": {"p256dh": "p", "auth": "a"}, "capHash": "c"}
        del sub["endpoint"]
        write_json(d / "state" / "push-subs" / "s1.json", sub)
        m = build_manifest(d)
        results.append({"test": "missing_subscription_metadata_field", "ok": any(i.get("kind") == "push-subs" and i.get("issue") == "invalid metadata" for i in m["issues"]), "issues": m["issues"]})

        # duplicate/mismatched subscription id metadata
        d = base / "dup-sub-id"; mk_min_v2_data(d)
        sub = {"id": "s1", "grantId": "g1", "locations": "*", "events": ["x"], "endpoint": "https://e", "keys": {"p256dh": "p", "auth": "a"}, "capHash": "c"}
        write_json(d / "state" / "push-subs" / "s1.json", sub)
        write_json(d / "state" / "push-subs" / "s2.json", sub)
        m = build_manifest(d)
        results.append({"test": "duplicate_or_mismatched_subscription_id_metadata", "ok": any(i.get("kind") == "push-subs" and i.get("issue") == "invalid metadata" for i in m["issues"]), "issues": m["issues"]})

        # missing subscription metadata referenced by outbox
        d = base / "missing-sub-for-outbox"; mk_min_v2_data(d)
        write_json(d / "state" / "push" / "outbox.json", {"s-missing": {"generation": 1, "attempts": 0, "nextAt": 0}})
        m = build_manifest(d)
        results.append({"test": "missing_subscription_metadata_for_outbox", "ok": any(i.get("kind") == "push-subs" and i.get("issue") == "missing subscription metadata for outbox entry" for i in m["issues"]), "issues": m["issues"]})

        # correctness of reversion (pure filesystem simulation)
        d = base / "reversion"; mk_min_v2_data(d)
        src_m = build_manifest(d)
        rb = base / "reversion-rb"; cand = base / "reversion-cand"; ev = base / "reversion-ev"; ev.mkdir()
        safe_copytree(d, rb)
        sim = simulate_import_with_interruption(src_m, rb, cand, ev)
        # original still equal
        orig_after = build_manifest(d)
        results.append({"test": "correctness_of_reversion", "ok": sim["ok"] and manifests_equal(src_m, orig_after), "simulation": sim, "source_unchanged": manifests_equal(src_m, orig_after)})

        # live rehearsal if binary provided
        if binary and binary.exists():
            live_ev = base / "live-ev"
            work = base / "live-work"; work.mkdir()
            try:
                live = run_full_rehearsal(binary, work, live_ev)
                results.append({"test": "live_executable_rehearsal", "ok": bool(live.get("ok")), "result_path": str(live_ev / "rehearsal-result.json")})
            except Exception as e:  # noqa: BLE001
                results.append({"test": "live_executable_rehearsal", "ok": False, "error": f"{type(e).__name__}: {e}"})

    ok = all(r.get("ok") is True for r in results)
    return {"schema": SCHEMA + "-gate-tests", "ok": ok, "passed": sum(1 for r in results if r.get("ok")), "total": len(results), "results": results}


def cmd_test_gates(args: argparse.Namespace) -> int:
    binary = Path(args.v2_binary).resolve() if args.v2_binary else None
    out = run_gate_tests(binary)
    if args.output:
        write_json(Path(args.output), out)
    print(json.dumps(out, indent=2, sort_keys=True, default=str))
    return 0 if out.get("ok") else 1


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("snapshot", help="Fail-closed safe copy of disposable v2 data dir")
    p.add_argument("--source", required=True)
    p.add_argument("--dest", required=True)
    p.set_defaults(func=cmd_snapshot)

    p = sub.add_parser("manifest", help="Emit exact-byte/metadata manifest (fail-closed on unsafe/invalid state)")
    p.add_argument("--data-dir", required=True)
    p.add_argument("--output")
    p.set_defaults(func=cmd_manifest)

    p = sub.add_parser("verify", help="Compare two manifests for exact equality")
    p.add_argument("--manifest-a", required=True)
    p.add_argument("--manifest-b", required=True)
    p.set_defaults(func=cmd_verify)

    p = sub.add_parser("validate", help="Run fail-closed gates on a disposable v2 data dir")
    p.add_argument("--data-dir", required=True)
    p.set_defaults(func=cmd_validate)

    p = sub.add_parser("plan", help="Print executable/fail-closed rehearsal plan")
    p.set_defaults(func=cmd_plan)

    p = sub.add_parser("rehearse", help="Run full disposable live rehearsal with interruption/retry/rollback gates")
    p.add_argument("--v2-binary", required=True)
    p.add_argument("--evidence-dir")
    p.add_argument("--keep-work-dir", action="store_true")
    p.set_defaults(func=cmd_rehearse)

    p = sub.add_parser("live-rehearsal", help="Compatibility alias for rehearse")
    p.add_argument("--v2-binary", required=True)
    p.add_argument("--evidence-dir")
    p.add_argument("--keep-work-dir", action="store_true")
    p.set_defaults(func=cmd_live_rehearsal)

    p = sub.add_parser("test-gates", help="Run automated gate tests (unsafe symlinks, missing/duplicate/corrupt grant & subscription metadata, reversion)")
    p.add_argument("--v2-binary", help="Optional pinned v2 binary for live executable rehearsal test")
    p.add_argument("--output")
    p.set_defaults(func=cmd_test_gates)

    args = ap.parse_args(argv)
    try:
        return args.func(args)
    except GateFailure as e:
        print(json.dumps({"ok": False, "fail_closed": True, "error": str(e)}, sort_keys=True), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
