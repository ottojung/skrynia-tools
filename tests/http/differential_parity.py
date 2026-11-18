#!/usr/bin/env python3
"""Black-box differential parity orchestration for pinned v2 vs v3+facade.

Compares wire-level status, selected headers, and bodies for inventory cases.
Default mode is read-only public probes. Mutation/auth cases require explicit
disposable-target configuration and are never run against production data.

Inventory schema compatibility target: skrynia-tools.v2-compat-inventory.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from typing import Any

DEFAULT_PUBLIC_IDS = {"health", "version", "push_vapid", "store_get", "store_head"}


@dataclass
class Response:
    status: int
    headers: dict[str, str]
    body: bytes
    error: str | None = None


def load_inventory(path: str) -> dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if "endpoints" not in data or not isinstance(data["endpoints"], list):
        raise ValueError("inventory missing endpoints list")
    return data


def norm_headers(headers: Any) -> dict[str, str]:
    out: dict[str, str] = {}
    for k, v in headers.items():
        out[str(k).lower()] = str(v)
    return out


def do_request(base: str, method: str, path: str, headers: dict[str, str] | None = None,
               body: bytes | None = None, timeout: float = 10.0) -> Response:
    url = base.rstrip("/") + path
    req = urllib.request.Request(url, data=body, method=method)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
            return Response(resp.status, norm_headers(resp.headers), raw)
    except urllib.error.HTTPError as e:
        raw = e.read() if hasattr(e, "read") else b""
        return Response(int(e.code), norm_headers(getattr(e, "headers", {}) or {}), raw)
    except Exception as e:  # noqa: BLE001 - report transport as evidence
        return Response(0, {}, b"", error=f"{type(e).__name__}: {e}")


def body_for_compare(body: bytes, content_type: str) -> Any:
    if "json" in content_type:
        try:
            return json.loads(body.decode("utf-8"))
        except Exception:
            return {"_raw_utf8": body.decode("utf-8", errors="replace")}
    try:
        return body.decode("utf-8")
    except Exception:
        return {"_bytes_hex": body.hex()}


def compare(name: str, a: Response, b: Response, header_keys: list[str]) -> tuple[bool, str]:
    if a.error or b.error:
        ok = a.error == b.error and a.status == b.status
        return ok, f"{name}: transport baseline={a.error or a.status} candidate={b.error or b.status}"
    diffs = []
    if a.status != b.status:
        diffs.append(f"status {a.status}!={b.status}")
    for hk in header_keys:
        k = hk.lower()
        av, bv = a.headers.get(k), b.headers.get(k)
        if av != bv:
            diffs.append(f"header {k} {av!r}!={bv!r}")
    act = a.headers.get("content-type", "")
    bct = b.headers.get("content-type", "")
    ba = body_for_compare(a.body, act)
    bb = body_for_compare(b.body, bct)
    if ba != bb:
        diffs.append(f"body differs (baseline={len(a.body)}B candidate={len(b.body)}B)")
    if diffs:
        return False, f"{name}: FAIL " + "; ".join(diffs)
    return True, f"{name}: PASS status={a.status}"


def public_case(case_id: str, baseline: str, candidate: str, missing_key: str) -> tuple[bool, str]:
    if case_id in ("health", "version", "push_vapid"):
        path = {"health": "/health", "version": "/version", "push_vapid": "/push/vapid"}[case_id]
        a = do_request(baseline, "GET", path)
        b = do_request(candidate, "GET", path)
        return compare(case_id, a, b, ["content-type"])
    if case_id == "store_get":
        path = f"/store/{missing_key}"
        a = do_request(baseline, "GET", path)
        b = do_request(candidate, "GET", path)
        # Compare wire behavior for known-missing object; status/header/body parity matters.
        return compare(case_id, a, b, ["content-type", "cache-control", "x-content-type-options"])
    if case_id == "store_head":
        path = f"/store/{missing_key}"
        a = do_request(baseline, "HEAD", path)
        b = do_request(candidate, "HEAD", path)
        return compare(case_id, a, b, ["content-type", "cache-control", "x-content-type-options"])
    return False, f"{case_id}: SKIP no public probe implemented"


def self_test() -> int:
    inv = {
        "schema": "skrynia-tools.v2-compat-inventory",
        "endpoints": [{"id": "health", "method": "GET", "path": "/health", "auth": "public", "success": [200], "negative": []}],
    }
    assert inv["endpoints"][0]["id"] == "health"
    a = Response(404, {"content-type": "application/json"}, b'{"error":"missing"}')
    b = Response(404, {"content-type": "application/json"}, b'{"error":"missing"}')
    ok, msg = compare("sample", a, b, ["content-type"])
    assert ok, msg
    b2 = Response(200, {"content-type": "application/json"}, b"{}")
    ok2, _ = compare("sample", a, b2, ["content-type"])
    assert not ok2
    print("self_test_ok")
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--inventory", help="Path to v2 compatibility inventory JSON")
    ap.add_argument("--baseline", default=os.environ.get("SKRYNIA_V2_BASE_URL", ""), help="Pinned v2 base URL")
    ap.add_argument("--candidate", default=os.environ.get("SKRYNIA_V3_FACADE_BASE_URL", ""), help="Candidate v3+facade base URL")
    ap.add_argument("--missing-key", default="__differential_parity_probe_missing__", help="Key expected absent for read probes")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--ids", default="", help="Comma-separated endpoint ids to probe (default: safe public subset)")
    ap.add_argument("--allow-unsafe", action="store_true", help="Reserved for future disposable-target mutation orchestration")
    args = ap.parse_args(argv)

    if args.self_test:
        return self_test()

    if args.inventory:
        inv = load_inventory(args.inventory)
        ids = [e.get("id") for e in inv.get("endpoints", []) if isinstance(e, dict)]
        print(f"inventory_loaded endpoints={len(ids)} schema={inv.get('schema')}")
    else:
        inv = {"endpoints": [{"id": i} for i in sorted(DEFAULT_PUBLIC_IDS)]}

    if not args.baseline or not args.candidate:
        print("no_targets: set --baseline/--candidate (or SKRYNIA_V2_BASE_URL/SKRYNIA_V3_FACADE_BASE_URL) for differential run")
        print("mode=inventory-check-only" if args.inventory else "mode=self-check-only")
        return 0

    selected = [s.strip() for s in args.ids.split(",") if s.strip()] if args.ids else sorted(DEFAULT_PUBLIC_IDS)
    results = []
    for cid in selected:
        if cid not in DEFAULT_PUBLIC_IDS:
            print(f"{cid}: SKIP requires auth/mutation/disposable setup (not run)")
            results.append((cid, None))
            continue
        ok, msg = public_case(cid, args.baseline, args.candidate, args.missing_key)
        print(msg)
        results.append((cid, ok))
    failed = [cid for cid, ok in results if ok is False]
    if failed:
        print(f"result=fail failed={','.join(failed)}")
        return 1
    print("result=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
