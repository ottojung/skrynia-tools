#!/usr/bin/env python3
"""Real-HTTP code-only A->B->A CAS rollback and forged/unready-code negative.

Drives the exact pinned native binary protocol through a real token-gated
Program0 and a generic kernel; no Node filesystem mock and no host-side Rust
eval stub. The two "code" blobs are tiny independently-built WASM guests that
write distinct markers ("CODE_A"/"CODE_B"), so a code-only rollback is proved
by the INVOKE output changing with the active pointer hash, not by a mocked
read.

Usage:
  python3 rollback_e2e.py SERVER KERNEL_P0_WASM ORDINARY_P1_WASM [PORT]

SERVER               pinned kernel binary (skrynia-server, commit c39bcbf)
KERNEL_P0_WASM       root token-gating Program0 wasm (sha256 8a03330d...)
ORDINARY_P1_WASM     ordinary nonzero cb-guest service (sha256 fc96db3d...)

Exit 0 only when every assertion holds. On a P1 whose activate path omits the
readiness seal check, the UNREADY_NEGATIVE step fails (RED) and exit is 1.
"""

import hashlib
import os
import socket
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path

SERVER, P0_WASM, P1_WASM = sys.argv[1:4]
PORT = int(sys.argv[4]) if len(sys.argv) > 4 else 17520
TOKEN = hashlib.sha256(b"gov-real-http-token").digest()
OP = hashlib.sha256(b"gov-operator-cap").digest()
ADMIN = hashlib.sha256(b"gov-admin-cap").digest()
OK, ERR = 0, 1

failures = []


def check(label, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + label + ("  " + detail if detail else ""))
    if not cond:
        failures.append(label)


def put_id(v):
    return struct.pack("<H", len(v)) + v


def put_blob(v):
    return struct.pack("<I", len(v)) + v


def u64le(n):
    return struct.pack("<Q", n)


def u16le(n):
    return struct.pack("<H", n)


def req(op, body):
    return bytes([1, op]) + body


def marker_wasm(marker):
    """Minimal WASM: skrynia_main(input_fd, output_fd) writes a fixed marker."""
    def uleb(n):
        out = bytearray()
        while True:
            b = n & 0x7F
            n >>= 7
            out.append(b | 0x80 if n else b)
            if not n:
                return bytes(out)

    def section(sid, payload):
        return bytes([sid]) + uleb(len(payload)) + payload

    def vec(items):
        return uleb(len(items)) + b"".join(items)

    t_write = bytes([0x60, 0x03, 0x7F, 0x7F, 0x7F, 0x01, 0x7F])
    t_main = bytes([0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F])
    types = section(1, vec([t_write, t_main]))
    imp = bytes([0x07]) + b"skrynia" + bytes([0x05]) + b"write" + bytes([0x00, 0x00])
    imports = section(2, vec([imp]))
    func = section(3, vec([uleb(1)]))
    mem = section(5, vec([bytes([0x00, 0x01])]))
    exp_mem = bytes([0x06]) + b"memory" + bytes([0x02, 0x00])
    exp_main = bytes([0x0C]) + b"skrynia_main" + bytes([0x00, 0x01])
    exports = section(7, vec([exp_mem, exp_main]))
    body = bytes([0x00, 0x20, 0x01, 0x41, 0x80, 0x08, 0x41]) + uleb(len(marker)) + \
        bytes([0x10, 0x00, 0x1A, 0x41, 0x00, 0x0B])
    code = section(10, vec([uleb(len(body)) + body]))
    data = section(11, vec([bytes([0x00, 0x41, 0x80, 0x08, 0x0B]) + uleb(len(marker)) + marker]))
    return b"\x00asm\x01\x00\x00\x00" + types + imports + func + mem + exports + code + data


def post(payload):
    body = b"\x01" + TOKEN + payload
    s = socket.create_connection(("127.0.0.1", PORT), 5)
    s.settimeout(20)
    s.sendall((f"POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {len(body)}\r\n"
               "Connection: close\r\n\r\n").encode() + body)
    raw = bytearray()
    try:
        while True:
            b = s.recv(65536)
            if not b:
                break
            raw.extend(b)
    except Exception as exc:  # noqa: BLE001 - harness diagnostics
        print("RECV_ERR", repr(exc))
    s.close()
    raw = bytes(raw)
    head, rb = raw.split(b"\r\n\r\n", 1) if b"\r\n\r\n" in raw else (raw, b"")
    if b"chunked" in head.lower():
        out = bytearray()
        i = 0
        while True:
            j = rb.find(b"\r\n", i)
            if j < 0:
                break
            n = int(rb[i:j].split(b";", 1)[0], 16)
            if n == 0:
                break
            out.extend(rb[j + 2:j + 2 + n])
            i = j + 2 + n + 2
        rb = bytes(out)
    return head.split(b"\r\n", 1)[0].decode("latin1", "replace"), rb


def response(payload):
    """Return (http_status, protocol_status, body)."""
    http, raw = post(payload)
    if len(raw) < 6 or raw[0] != 1:
        return http, None, raw
    return http, raw[1], raw[6:6 + struct.unpack("<I", raw[2:6])[0]]


def active(body):
    ver, live, h, rev = body[0], body[1], body[2:34], struct.unpack("<Q", body[34:42])[0]
    return ver, live, h, rev


def main():
    code_a, code_b = marker_wasm(b"CODE_A"), marker_wasm(b"CODE_B")
    hash_a = hashlib.sha256(code_a).digest()
    hash_b = hashlib.sha256(code_b).digest()
    with tempfile.TemporaryDirectory(prefix="gov-rollback-", dir="/tmp/opencode") as td:
        d = Path(td)
        op, us = d / "operator", d / "user"
        (op / "security/program").mkdir(parents=True)
        (op / "programs").mkdir(parents=True)
        (us / "programs").mkdir(parents=True)
        (op / "security/program0.sha256").write_bytes(hashlib.sha256(TOKEN).digest())
        (op / "programs/root.wasm").write_bytes(Path(P1_WASM).read_bytes())
        (us / "programs/operator.cap").write_bytes(OP)
        meta = struct.pack("<II", 0xFFF, len(b"programs")) + b"programs"
        (op / "security/program/programs%2Froot.wasm").write_bytes(meta)
        env = {**os.environ, "SKRYNIA_ROOT_WASM": P0_WASM, "SKRYNIA_OPERATOR_DIR": str(op),
               "SKRYNIA_USER_DIR": str(us), "SKRYNIA_BIND": f"127.0.0.1:{PORT}"}
        log = (d / "server.log").open("wb")
        proc = subprocess.Popen([SERVER], env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if proc.poll() is not None:
                    print("SERVER_EXIT", proc.returncode)
                    print((d / "server.log").read_text(errors="replace")[-3000:])
                    return 2
                try:
                    socket.create_connection(("127.0.0.1", PORT), 0.2).close()
                    break
                except OSError:
                    time.sleep(0.2)

            http, st, body = response(req(15, put_id(OP) + put_id(ADMIN) + put_blob(code_a)))
            check("REGISTER_200", http.startswith("HTTP/1.1 200"), http)
            check("REGISTER_OK", st == OK, f"st={st}")
            iid = body

            _, st, body = response(req(12, put_id(iid)))
            check("GET_ACTIVE_INITIAL", st == OK and active(body)[2] == hash_a and active(body)[3] == 1,
                  f"rev={active(body)[3] if len(body) >= 42 else '?'}")
            rev = active(body)[3]

            _, st, body = response(req(9, put_id(ADMIN) + put_blob(code_b)))
            check("STAGE_B", st == OK and body == hash_b, f"hash_b={body.hex()[:8]}")
            check("IMMUTABLE_DISTINCT_HASH", hash_a != hash_b)

            # NEGATIVE: staged-but-unsealed code must not be activatable.
            _, st, body = response(req(11, put_id(ADMIN) + u64le(rev) + hash_b))
            check("UNREADY_NEGATIVE", st == ERR, f"st={st} body={body.hex()}")

            _, st, _ = response(req(10, put_id(ADMIN) + hash_b))
            check("SEAL_B", st == OK)

            _, st, body = response(req(11, put_id(ADMIN) + u64le(rev) + hash_b))
            check("ACTIVATE_A_TO_B", st == OK and active(body)[2] == hash_b and active(body)[3] == rev + 1,
                  f"rev={active(body)[3] if len(body) >= 42 else '?'}")

            _, st, body = response(req(3, put_id(iid) + put_blob(b"ignored")))
            check("INVOKE_RUNS_B", st == OK and body.endswith(b"CODE_B"), f"body={body[-16:]!r}")

            # Re-stage original code: immutable content-addressed blob dedups.
            _, st, body = response(req(9, put_id(ADMIN) + put_blob(code_a)))
            check("RESTAGE_A_SAME_HASH", st == OK and body == hash_a)

            _, st, _ = response(req(10, put_id(ADMIN) + hash_a))
            check("SEAL_A", st == OK)

            _, st, body = response(req(11, put_id(ADMIN) + u64le(rev + 1) + hash_a))
            check("ROLLBACK_B_TO_A", st == OK and active(body)[2] == hash_a and active(body)[3] == rev + 2,
                  f"rev={active(body)[3] if len(body) >= 42 else '?'}")

            _, st, body = response(req(3, put_id(iid) + put_blob(b"ignored")))
            check("INVOKE_RUNS_A_AFTER_ROLLBACK", st == OK and body.endswith(b"CODE_A"), f"body={body[-16:]!r}")

            _, st, _ = response(req(11, put_id(ADMIN) + u64le(rev) + hash_b))
            check("STALE_CAS_NEGATIVE", st == ERR)

            forged = hashlib.sha256(b"never-staged-forged-code").digest()
            _, st, _ = response(req(11, put_id(ADMIN) + u64le(rev + 2) + forged))
            check("FORGED_HASH_NEGATIVE", st == ERR)

            _, st, body = response(req(13, put_id(iid) + bytes([0]) + u16le(256)))
            n = struct.unpack("<H", body[:2])[0] if len(body) >= 2 else 0
            hashes = [body[2 + i * 32:34 + i * 32] for i in range(n)]
            check("HISTORY_RETAINS_ALL_RELEASES", st == OK and hash_a in hashes and hash_b in hashes,
                  f"count={n}")

            _, st, body = response(req(12, put_id(iid)))
            check("FINAL_ACTIVE_MONOTONIC", st == OK and active(body)[2] == hash_a and active(body)[3] == rev + 2,
                  f"rev={active(body)[3] if len(body) >= 42 else '?'}")
            return 1 if failures else 0
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except Exception:  # noqa: BLE001
                proc.kill()
                proc.wait(timeout=5)
            log.close()


if __name__ == "__main__":
    sys.exit(main())
