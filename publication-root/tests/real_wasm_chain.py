#!/usr/bin/env python3
"""Real compiled-stack publication lifecycle driven by the produced root WASM.

Not a hand-written stand-in: the real kernel executes the compiled
skrynia-publication-root.wasm, which imports syscall.publish/unpublish and
writes the private mailbox itself. External artifacts are supplied by path:

  --kernel   real skrynia-server (post /syscalls_list discovery)
  --provider real skrynia-publish-provider binary
  --root     compiled skrynia_publication_root.wasm

All state is disposable; no production config, DNS or credentials are used.
"""
from __future__ import annotations
import argparse
import concurrent.futures
import hashlib
import http.server
import json
import os
import pathlib
import shutil
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request

OP_PUBLISH = 1
OP_UNPUBLISH = 2
RELEASE = "a" * 64
OLD_RELEASE = "b" * 64
NGINX_CANDIDATES = (
    "/gnu/store/db9fbd8i7dxn3k2rs7rll52d1gxx4wsy-nginx-1.28.0/sbin/nginx",
)


class CheckFailure(AssertionError):
    pass


def free_port() -> int:
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    port = sock.getsockname()[1]
    sock.close()
    return port


def request(url: str, body: bytes | None, method: str, timeout: int = 15):
    req = urllib.request.Request(url, data=body, method=method,
                                 headers={"Content-Type": "application/octet-stream"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read()


def post(url: str, body: bytes) -> tuple[int, bytes]:
    return request(url, body, "POST")


def get(port: int, path: str) -> tuple[int, bytes]:
    return request(f"http://127.0.0.1:{port}{path}", None, "GET", 5)


def framed(token: bytes, op: int, path: str) -> bytes:
    raw = path.encode("utf-8")
    assert 0 < len(raw) <= 4096
    return b"\x01" + token + bytes([op]) + struct.pack("<I", len(raw)) + raw


CALL_FAILURES: list[tuple[int, int, bytes]] = []


def wasm_call(url: str, token: bytes, op: int, path: str) -> int | None:
    status, body = post(url, framed(token, op, path))
    if status != 200 or len(body) != 4:
        CALL_FAILURES.append((status, len(body), body[:80]))
        return None
    return struct.unpack("<i", body)[0]


def write_atomic(path: str, payload: bytes) -> None:
    tmp = path + ".next"
    with open(tmp, "wb") as handle:
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def seed_private(source: str, release: str, payload: bytes) -> str:
    rel = f"apps/qual/releases/{release}/asset.js"
    path = os.path.join(source, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    pathlib.Path(path).write_bytes(payload)
    return rel


def find_nginx(explicit: str) -> str:
    for candidate in ([explicit] if explicit else []) + list(NGINX_CANDIDATES) + [shutil.which("nginx") or ""]:
        if candidate and os.access(candidate, os.X_OK):
            return candidate
    raise CheckFailure("nginx binary not found")


class Nginx:
    def __init__(self, binary: str, workdir: str, port: int, root: str):
        self.binary, self.workdir, self.port, self.root = binary, workdir, port, root
        self.conf = os.path.join(workdir, "nginx.conf")
        self.pid = None

    def start(self) -> None:
        pathlib.Path(self.conf).write_text(f"""worker_processes 1;
pid {self.conf}.pid;
error_log {self.conf}.error.log info;
events {{ worker_connections 64; }}
http {{
  default_type application/octet-stream;
  access_log off;
  server {{
    listen {self.port};
    server_name localhost;
    root {self.root};
    disable_symlinks on;
    location /apps/ {{ disable_symlinks off; try_files $uri =404; }}
    location /releases/ {{ disable_symlinks off; try_files $uri =404; }}
    location = /active.json {{ add_header Cache-Control "no-store" always; try_files $uri =404; }}
    location / {{ try_files $uri =404; }}
  }}
}}
""")
        check = subprocess.run([self.binary, "-t", "-p", self.workdir, "-c", self.conf],
                               capture_output=True, text=True)
        require(check.returncode == 0, f"nginx -t failed: {check.stderr.strip()}")
        self.pid = subprocess.Popen(
            [self.binary, "-g", "daemon off;", "-p", self.workdir, "-c", self.conf],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).pid
        for _ in range(100):
            try:
                get(self.port, "/")
                return
            except OSError:
                time.sleep(0.05)
        raise CheckFailure("nginx did not become ready")

    def stop(self) -> None:
        if self.pid is not None:
            try:
                os.kill(self.pid, signal.SIGTERM)
                os.waitpid(self.pid, 0)
            except OSError:
                pass
            self.pid = None


class Process:
    def __init__(self, binary: str, port: int, env: dict, ready: str, log: str):
        self.binary, self.port, self.env = binary, port, env
        self.log_path, self.ready = log, ready
        self.proc = None
        self.log = None

    def start(self, workdir: str, name: str) -> None:
        self.log_path = os.path.join(workdir, f"{name}.log")
        self.log = open(self.log_path, "wb")
        self.proc = subprocess.Popen([self.binary], env=self.env, stdout=self.log,
                                     stderr=subprocess.STDOUT)
        for _ in range(240):
            if self.proc.poll() is not None:
                raise CheckFailure(f"{name} exited: " + open(self.log_path).read()[-800:])
            try:
                if request(self.ready, b"" if "provider" in name else None,
                           "GET" if "provider" in name else "POST")[0] == 200:
                    return
            except OSError:
                pass
            time.sleep(0.05)
        raise CheckFailure(f"{name} did not become ready")

    def stop(self) -> None:
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=5)
            except Exception:
                self.proc.kill()
                self.proc.wait(timeout=5)
        if self.log:
            self.log.close()


class SlowProviderProxy:
    """Transparent loopback forwarder that holds one marked provider call.

    The root has already CAS-claimed a shared-RAM slot and staged the mailbox
    ticket before this call reaches the provider. Holding that one call open
    while >256 other invocations finish proves the active slot is not reused
    and its staged request path is not clobbered by a peer.
    """

    def __init__(self, port: int, target: int, mailbox: str, timeout: int):
        self.target, self.mailbox, self.timeout = target, mailbox, timeout
        self.held = threading.Event()
        self.release = threading.Event()
        self.proxied = 0
        outer = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def forward(self):
                length = int(self.headers.get("Content-Length", "0"))
                body = self.rfile.read(length) if length else b""
                if (self.command == "POST" and len(body) == 7
                        and list(body[:3]) == [1, 1, 0x32]):
                    ticket = int.from_bytes(body[3:7], "little")
                    notice = os.path.join(outer.mailbox, f"{ticket:08x}")
                    if os.path.exists(notice) and b"overlap-hold" in pathlib.Path(notice).read_bytes():
                        outer.held.set()
                        outer.release.wait(timeout=outer.timeout)
                url = f"http://127.0.0.1:{outer.target}{self.path}"
                req = urllib.request.Request(
                    url, data=body if body else None, method=self.command,
                    headers={"Content-Type": "application/octet-stream"})
                try:
                    with urllib.request.urlopen(req, timeout=30) as response:
                        status, payload = response.status, response.read()
                except urllib.error.HTTPError as exc:
                    status, payload = exc.code, exc.read()
                outer.proxied += 1
                self.send_response(status)
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

            do_GET = forward
            do_POST = forward

        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler)

    def start(self) -> None:
        threading.Thread(target=self.server.serve_forever, daemon=True).start()

    def stop(self) -> None:
        self.release.set()
        self.server.shutdown()
        self.server.server_close()


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise CheckFailure(detail)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--kernel", required=True)
    ap.add_argument("--provider", required=True)
    ap.add_argument("--root", required=True)
    ap.add_argument("--nginx")
    ap.add_argument("--results", required=True)
    ap.add_argument("--overlap-total", type=int, default=0,
                    help="hold one request active across this many other publishes (>256)")
    ap.add_argument("--hold-seconds", type=int, default=180,
                    help="upper bound for the held provider call (released explicitly)")
    ap.add_argument("--overlap-workers", type=int, default=8,
                    help="concurrent peer publishes (kernel permits 16 in flight)")
    args = ap.parse_args()

    results = {"checks": {}, "artifacts": {}}
    for name, path in [("kernel", args.kernel), ("provider", args.provider), ("root", args.root)]:
        results["artifacts"][name] = {
            "path": path,
            "sha256": hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest(),
        }

    def record(name: str, ok: bool, detail="") -> None:
        results["checks"][name] = {"ok": bool(ok), "detail": str(detail)}
        print(("PASS" if ok else "FAIL"), name, detail, flush=True)

    tmp = tempfile.mkdtemp(prefix="skrynia-wasm-pub-")
    operator = os.path.join(tmp, "operator")
    nginx_root = os.path.join(tmp, "nginx-root")
    # Publication VM user root (mailbox) is disjoint from the source artifact root.
    pub_user = os.path.join(tmp, "publication-user")
    mailbox = os.path.join(pub_user, "publication-requests")
    source = os.path.join(tmp, "source-disk")
    for directory in (operator, pub_user, source, nginx_root, mailbox, os.path.join(tmp, "nginx")):
        os.makedirs(directory, exist_ok=True)
    token = hashlib.sha256(b"real-wasm-publication-test-token").digest()
    os.makedirs(os.path.join(operator, "security"), exist_ok=True)
    pathlib.Path(operator, "security", "program0.sha256").write_bytes(hashlib.sha256(token).digest())

    rel = seed_private(source, RELEASE, b"REAL WASM PUBLICATION BYTES")
    old_rel = seed_private(source, OLD_RELEASE, b"OLD RELEASE BYTES")
    active = "apps/qual/active.json"
    write_atomic(os.path.join(source, active), b'{"release":"' + OLD_RELEASE.encode() + b'"}')

    provider_port, kernel_port, nginx_port = free_port(), free_port(), free_port()
    proxy_port = free_port() if args.overlap_total else provider_port
    proxy = (SlowProviderProxy(proxy_port, provider_port, mailbox, args.hold_seconds)
             if args.overlap_total else None)
    provider = Process(args.provider, provider_port, {
        **os.environ,
        "PUBLICATION_PROVIDER_BIND": f"127.0.0.1:{provider_port}",
        "PATH_TO_PUBLICATION_REQUESTS": mailbox,
        "PATH_TO_SKRYNIA_DISK": source,
        "PATH_TO_NGINX_SKRYNIA_ROOT": nginx_root,
    }, f"http://127.0.0.1:{provider_port}/syscalls_list", "provider")
    kernel = Process(args.kernel, kernel_port, {
        **os.environ,
        "SKRYNIA_ROOT_WASM": args.root,
        "SKRYNIA_OPERATOR_DIR": operator,
        "SKRYNIA_USER_DIR": pub_user,
        "SKRYNIA_BIND": f"127.0.0.1:{kernel_port}",
        "SKRYNIA_SYSCALL_URL": f"http://127.0.0.1:{proxy_port}",
    }, f"http://127.0.0.1:{kernel_port}/version", "kernel")
    nginx = Nginx(find_nginx(args.nginx), os.path.join(tmp, "nginx"), nginx_port, nginx_root)
    try:
        if proxy is not None:
            proxy.start()
        provider.start(tmp, "provider")
        kernel.start(tmp, "kernel")
        nginx.start()
        url = kernel.url if hasattr(kernel, "url") else f"http://127.0.0.1:{kernel_port}/"
        syscalls = json.loads(request(f"http://127.0.0.1:{kernel_port}/version", None, "POST")[1])
        descriptors = syscalls.get("syscalls", [])
        # Target-ABI kernels advertise typed descriptors; legacy kernels
        # advertised bare names. Accept both shapes but require exact names.
        names = sorted(d["name"] if isinstance(d, dict) else d for d in descriptors)
        record("kernel_discovers_exact_names", names == ["publish", "unpublish"], names)
        record("kernel_discovers_typed_i32_signatures",
               bool(descriptors) and all(
                   isinstance(d, dict) and d.get("params") == ["i32"]
                   and d.get("results") == ["i32"] for d in descriptors))
        record("before_publish_inaccessible", get(nginx_port, "/" + rel)[0] == 404)

        record("wasm_publish_returns_zero", wasm_call(url, token, OP_PUBLISH, rel) == 0)
        status, body = get(nginx_port, "/" + rel)
        record("after_publish_exact_bytes", status == 200 and body == b"REAL WASM PUBLICATION BYTES", (status, body))
        record("mailbox_clean_after_publish", not os.listdir(mailbox))

        record("wasm_unpublish_returns_zero", wasm_call(url, token, OP_UNPUBLISH, rel) == 0)
        # Unpublish of an absent path is idempotent: no node, no conflict.
        record("after_unpublish_inaccessible",
               get(nginx_port, "/" + rel)[0] == 404
               and wasm_call(url, token, OP_UNPUBLISH, rel) == 0)
        record("source_file_intact", pathlib.Path(source, rel).read_bytes() == b"REAL WASM PUBLICATION BYTES")

        # Private roots are qualified by the K10 `operator/` and `user/`
        # tokens; the unqualified legacy names are kept so no spelling of a
        # private root can reach the mailbox or the provider.
        bad = [b"security/program0.sha256", b"programs/root.wasm", b"publication-requests/00000001",
               b"apps/../../etc/passwd", b"/apps/qual/x", b"apps/qual/\0x",
               b"user/publication-requests/00000001", b"user/security/program0.sha256",
               b"user/programs/operator.cap", b"user/private/key", b"user/tokens/issuer",
               b"operator/security/program0.sha256", b"operator/programs/root.wasm"]
        record("private_paths_rejected_before_import",
               all(wasm_call(url, token, OP_PUBLISH, p.decode("latin1")) == -1 for p in bad))
        record("wrong_token_denied", wasm_call(url, b"\x00" * 32, OP_PUBLISH, rel) == -4)
        missing = f"apps/qual/releases/{RELEASE}/missing.js"
        # A missing source directory is not silently created and served: the
        # provider reports INVALID_SOURCE just like a missing final file.
        missing_dir = f"apps/qual/releases/{RELEASE}/missing-dir/x.js"
        provider_code = wasm_call(url, token, OP_PUBLISH, missing)
        provider_dir_code = wasm_call(url, token, OP_PUBLISH, missing_dir)
        record("provider_failure_passthrough",
               provider_code == 3 and provider_dir_code == 3,
               (provider_code, provider_dir_code))
        # An existing non-symlink directory at the destination name is a
        # conflict: it must survive unchanged and never be turned into a link.
        conflict = f"apps/qual/releases/{RELEASE}/dir-conflict.js"
        pathlib.Path(source, conflict).write_bytes(b"CONFLICT-SOURCE")
        os.makedirs(os.path.join(nginx_root, conflict), exist_ok=True)
        conflict_code = wasm_call(url, token, OP_PUBLISH, conflict)
        conflict_path = os.path.join(nginx_root, conflict)
        record("failed_publish_creates_no_public_node",
               get(nginx_port, "/" + missing)[0] == 404
               and get(nginx_port, "/" + missing_dir)[0] == 404
               and conflict_code == 4
               and os.path.isdir(conflict_path)
               and not os.path.islink(conflict_path),
               (conflict_code, conflict))
        record("mailbox_clean_after_failures", not os.listdir(mailbox))

        paths = [f"apps/qual/releases/{RELEASE}/part{i}.js" for i in range(16)]
        for p in paths:
            pathlib.Path(source, p).write_bytes(p.encode())
        with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
            codes = list(pool.map(lambda p: wasm_call(url, token, OP_PUBLISH, p), paths))
        record("concurrent_wasm_publications", codes == [0] * 16, codes)
        record("concurrent_bytes_exact", all(get(nginx_port, "/" + p) == (200, p.encode()) for p in paths))
        record("mailbox_clean_after_concurrency", not os.listdir(mailbox))

        if proxy is not None:
            total = args.overlap_total
            held_path = f"apps/qual/releases/{RELEASE}/overlap-hold.js"
            pathlib.Path(source, held_path).write_bytes(b"OVERLAP-HOLD-BYTES")
            overlap = []
            for i in range(total):
                p = f"apps/qual/releases/{RELEASE}/overlap{i}.js"
                payload = f"OVERLAP-{i}".encode()
                pathlib.Path(source, p).write_bytes(payload)
                overlap.append((p, payload))
            held_result: dict[str, int | None] = {}

            def held_call() -> None:
                held_result["code"] = wasm_call(url, token, OP_PUBLISH, held_path)

            thread = threading.Thread(target=held_call, daemon=True)
            thread.start()
            observed = proxy.held.wait(timeout=60)
            codes: list[int | None] = []
            if observed:
                with concurrent.futures.ThreadPoolExecutor(max_workers=args.overlap_workers) as pool:
                    codes = list(pool.map(lambda item: wasm_call(url, token, OP_PUBLISH, item[0]), overlap))
            proxy.release.set()
            thread.join(timeout=90)
            record("overlap_hold_observed", observed)
            record("overlap_gt_256_total_invocations",
                   total > 256 and len(codes) == total and not thread.is_alive(),
                   f"total={total} completed={len(codes)}")
            record("overlap_active_request_succeeded",
                   held_result.get("code") == 0
                   and get(nginx_port, "/" + held_path) == (200, b"OVERLAP-HOLD-BYTES"),
                   held_result.get("code"))
            record("overlap_all_peer_codes_zero", all(code == 0 for code in codes),
                   {"codes": [code for code in codes if code != 0][:8],
                    "http_failures": CALL_FAILURES[:4]})
            record("overlap_peer_bytes_exact",
                   all(get(nginx_port, "/" + p) == (200, payload) for p, payload in overlap))
            record("overlap_mailbox_clean", not os.listdir(mailbox))

        # Active pointer is published ONCE as a permanent symlink. A-B-A switches
        # rewrite only its private source bytes atomically; the public symlink
        # is never unlink/republish, so the switch is a single CAS-visible rename.
        write_atomic(os.path.join(source, active), b'{"release":"' + OLD_RELEASE.encode() + b'"}')
        record("active_published_once", wasm_call(url, token, OP_PUBLISH, active) == 0)
        switch_ok = True
        for version in (RELEASE, OLD_RELEASE, RELEASE):
            write_atomic(os.path.join(source, active), b'{"release":"' + version.encode() + b'"}')
            status, body = get(nginx_port, "/" + active)
            switch_ok &= status == 200 and version.encode() in body
        record("cas_aba_version_switch_follows_pointer", switch_ok)
        record("both_releases_retained",
               pathlib.Path(source, old_rel).exists() and pathlib.Path(source, rel).exists())
        record("old_release_asset_unserved", get(nginx_port, "/" + old_rel)[0] == 404)
    finally:
        nginx.stop()
        kernel.stop()
        provider.stop()
        if proxy is not None:
            proxy.stop()
        results["workdir"] = tmp
        pathlib.Path(args.results).write_text(json.dumps(results, indent=2))
        shutil.rmtree(tmp, ignore_errors=True)
    failed = [name for name, check in results["checks"].items() if not check["ok"]]
    print("ALL_OK=" + str(not failed), "FAILED=" + repr(failed))
    return 1 if failed else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        # Surface the concrete failure in CI check-run annotations so an
        # early startup error is diagnosable without private job logs.
        print(f"::error::real_wasm_chain failure: {exc}", flush=True)
        raise
