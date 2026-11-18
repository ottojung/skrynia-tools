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
import json
import os
import shutil
import sys
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


def cmd_plan(_: argparse.Namespace) -> int:
    plan = {
        "schema": "skrynia-v3-migration-rehearsal-plan",
        "version": 1,
        "production_mutation": False,
        "steps": [
            "Select pinned v2 source commit/build and disposable source data directory copy (never production).",
            "Snapshot source copy to rollback-copy and candidate-import-copy; record manifest for each.",
            "Verify rollback-copy manifest == source-copy manifest before any candidate import.",
            "When v3 kernel+Program0+compat facade candidate is available: import from candidate-import-copy into isolated v3 data dir per approved format/procedure.",
            "Produce post-import observable manifest from v3/facade (objects/metadata/grants/subscriptions/pointers as representable) and diff against source manifest.",
            "Run differential read/write probes on imported disposable instance; record deviations.",
            "Rollback validation: restart original v2 from untouched rollback-copy and verify service/manifest unchanged; discard v3 candidate data dir.",
            "Document trigger conditions, exact commands, artifacts, and sign-off evidence.",
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
    args = ap.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
