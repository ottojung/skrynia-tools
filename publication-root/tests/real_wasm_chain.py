#!/usr/bin/env python3
"""Real compiled-stack publication lifecycle driven by the produced root WASM.

This is not a hand-written stand-in: the kernel executes the actual
skrynia-publication-root.wasm, which imports syscall.publish/unpublish and
writes the private mailbox itself. External artifacts are supplied by path:

  --kernel     real skrynia-server (post /syscalls_list discovery)
  --provider   real skrynia-publish-provider binary
  --root       compiled skrynia_publication_root.wasm
  --apps-harness  skrynia-apps qualification/publication-isolation directory

All state is disposable; no production config, DNS or credentials are used.
"""
from __future__ import annotations
import argparse
import concurrent.futures
import hashlib
import importlib.util
import json
import os
import pathlib
import shutil
import struct
import sys
import tempfile
import time
import urllib.error
import urllib.request

OP_PUBLISH = 1
OP_UNPUBLISH = 2
RELEASE = "a" * 64
OLD_RELEASE = "b" * 64


def load(path: str, name: str):
    spec = importlib.util.spec_from_file_location(name, pathlib.Path(path) / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def post(url: str, body: bytes, timeout: int = 15):
    req = urllib.request.Request(url, data=body, method="POST",
                                 headers={"Content-Type": "application/octet-stream"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read()


def framed(token: bytes, op: int, path: str) -> bytes:
    raw = path.encode("utf-8")
    assert 0 < len(raw) <= 4096
    return b"\x01" + token + bytes([op]) + struct.pack("<I", len(raw)) + raw


def wasm_call(url: str, token: bytes, op: int, path: str) -> int | None:
    status, body = post(url, framed(token, op, path))
    if status != 200 or len(body) != 4:
        return None
    return struct.unpack("<i", body)[0]


def get(port: int, path: str):
    req = urllib.request.Request(f"http://127.0.0.1:{port}{path}", method="GET")
    try:
        with urllib.request.urlopen(req, timeout=5) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, b""


def write_atomic(path: str, payload: bytes) -> None:
    tmp = path + ".next"
    with open(tmp, "wb") as handle:
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--kernel", required=True)
    ap.add_argument("--provider", required=True)
    ap.add_argument("--root", required=True)
    ap.add_argument("--apps-harness", required=True)
    ap.add_argument("--nginx")
    ap.add_argument("--results", required=True)
    args = ap.parse_args()

    fullchain = load(args.apps_harness, "fullchain")
    realchain = load(args.apps_harness, "realchain")
    results = {"checks": {}, "artifacts": {}}
    for name, path in [("kernel", args.kernel), ("provider", args.provider), ("root", args.root)]:
        results["artifacts"][name] = {
            "path": path,
            "sha256": hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest(),
        }

    def record(name: str, ok: bool, detail="") -> None:
        results["checks"][name] = {"ok": bool(ok), "detail": str(detail)}
        print(("PASS" if ok else "FAIL"), name, detail, flush=True)

    tmp = tempfile.mkdtemp(prefix="skrynia-wasm-pub-", dir="/tmp/opencode")
    operator = os.path.join(tmp, "operator")
    private = os.path.join(tmp, "private-disk")
    nginx_root = os.path.join(tmp, "nginx-root")
    mailbox = os.path.join(private, "publication-requests")
    for directory in (operator, private, nginx_root, mailbox, os.path.join(tmp, "nginx")):
        os.makedirs(directory, exist_ok=True)
    token = hashlib.sha256(b"real-wasm-publication-test-token").digest()
    os.makedirs(os.path.join(operator, "security"), exist_ok=True)
    pathlib.Path(operator, "security", "program0.sha256").write_bytes(hashlib.sha256(token).digest())

    rel = fullchain.seed_private(private, "qual", RELEASE, b"REAL WASM PUBLICATION BYTES")
    old_rel = fullchain.seed_private(private, "qual", OLD_RELEASE, b"OLD RELEASE BYTES")
    active = "apps/qual/active.json"
    write_atomic(os.path.join(private, active), b'{"release":"' + OLD_RELEASE.encode() + b'"}')

    provider_port = fullchain.free_port()
    kernel_port = fullchain.free_port()
    nginx_port = fullchain.free_port()
    provider = realchain.RealProviderProc(args.provider, provider_port, private, mailbox, nginx_root)
    kernel = realchain.KernelProc(args.kernel, args.root, operator, private,
                                  f"http://127.0.0.1:{provider_port}", kernel_port)
    nginx = fullchain.Nginx(fullchain.find_nginx(args.nginx), os.path.join(tmp, "nginx"),
                            nginx_port, nginx_root, "on", "off")
    try:
        provider.start(tmp)
        kernel.start(tmp, "wasm-kernel")
        nginx.start()
        url = kernel.url + "/"
        record("kernel_discovers_provider", "publish" in kernel.version().get("syscalls", []))
        record("before_publish_inaccessible", get(nginx_port, "/" + rel)[0] == 404)

        code = wasm_call(url, token, OP_PUBLISH, rel)
        record("wasm_publish_returns_zero", code == 0, code)
        status, body = get(nginx_port, "/" + rel)
        record("after_publish_exact_bytes", status == 200 and body == b"REAL WASM PUBLICATION BYTES", (status, body))
        record("mailbox_clean_after_publish", not os.listdir(mailbox))

        code = wasm_call(url, token, OP_UNPUBLISH, rel)
        record("wasm_unpublish_returns_zero", code == 0, code)
        record("after_unpublish_inaccessible", get(nginx_port, "/" + rel)[0] == 404)
        record("source_file_intact", pathlib.Path(private, rel).read_bytes() == b"REAL WASM PUBLICATION BYTES")

        bad = [b"security/program0.sha256", b"programs/root.wasm", b"publication-requests/00000001",
               b"apps/../../etc/passwd", b"/apps/qual/x", b"apps/qual/\0x"]
        record("private_paths_rejected_before_import",
               all(wasm_call(url, token, OP_PUBLISH, p.decode("latin1")) == -1 for p in bad))
        record("wrong_token_denied", wasm_call(url, b"\x00" * 32, OP_PUBLISH, rel) == -4)
        record("mailbox_clean_after_failures", not os.listdir(mailbox))

        # Concurrent independent publications through the real WASM root.
        paths = [f"apps/qual/releases/{RELEASE}/part{i}.js" for i in range(16)]
        for p in paths:
            pathlib.Path(private, p).write_bytes(p.encode())
        with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
            codes = list(pool.map(lambda p: wasm_call(url, token, OP_PUBLISH, p), paths))
        record("concurrent_wasm_publications", codes == [0] * 16, codes)
        record("concurrent_bytes_exact",
               all(get(nginx_port, "/" + p) == (200, p.encode()) for p in paths))
        record("mailbox_clean_after_concurrency", not os.listdir(mailbox))

        # Version switch and CAS A-B-A: revoke before replacing the mutable
        # private pointer, publish the chosen immutable target, then restore.
        switch_ok = True
        for version, payload in [(OLD_RELEASE, b"OLD RELEASE BYTES"), (RELEASE, b"REAL WASM PUBLICATION BYTES"),
                                 (OLD_RELEASE, b"OLD RELEASE BYTES")]:
            write_atomic(os.path.join(private, active), b'{"release":"' + version.encode() + b'"}')
            switch_ok &= wasm_call(url, token, OP_UNPUBLISH, active) == 0
            switch_ok &= wasm_call(url, token, OP_PUBLISH, active) == 0
            status, body = get(nginx_port, "/" + active)
            switch_ok &= status == 200 and version.encode() in body
        record("cas_aba_version_switch_follows_pointer", switch_ok)
        record("old_release_asset_unserved", get(nginx_port, "/" + old_rel)[0] == 404)
    finally:
        nginx.stop()
        kernel.stop()
        provider.stop()
        results["workdir"] = tmp
        pathlib.Path(args.results).write_text(json.dumps(results, indent=2))
        shutil.rmtree(tmp, ignore_errors=True)
    failed = [name for name, check in results["checks"].items() if not check["ok"]]
    print("ALL_OK=" + str(not failed), "FAILED=" + repr(failed))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
