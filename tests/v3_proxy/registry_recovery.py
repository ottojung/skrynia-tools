#!/usr/bin/env python3
"""Compiled ordinary-service restart and filesystem-write recovery."""
import argparse
from contextlib import closing, contextmanager
import hashlib
import http.client
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time


def sha(value):
    return hashlib.sha256(value).digest()


def ident(value):
    return struct.pack('<H', len(value)) + value


def blob(value):
    return struct.pack('<I', len(value)) + value


@contextmanager
def server(args, base, port):
    env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(args.p0.resolve()),
           'SKRYNIA_OPERATOR_DIR': str(base / 'operator'),
           'SKRYNIA_USER_DIR': str(base / 'user'),
           'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    env.pop('SKRYNIA_SYSCALL_URL', None)
    with (base / 'server.log').open('ab') as log:
        proc = subprocess.Popen([str(args.kernel.resolve())], env=env,
                                stdout=log, stderr=subprocess.STDOUT)
        try:
            for _ in range(150):
                assert proc.poll() is None, 'fixture kernel exited'
                try:
                    with socket.create_connection(('127.0.0.1', port), 0.2):
                        break
                except OSError:
                    time.sleep(0.2)
            else:
                raise RuntimeError('fixture kernel did not listen')
            yield proc
        finally:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)


def run(args, base):
    operator, user = base / 'operator', base / 'user'
    token, op_cap, admin = (os.urandom(32) for _ in range(3))
    iid = sha(admin)
    (operator / 'security').mkdir(parents=True)
    (operator / 'programs').mkdir()
    (operator / 'security/program0.sha256').write_bytes(sha(token))
    (operator / 'programs/root.wasm').write_bytes(args.p1.read_bytes())
    programs = user / 'programs'
    for name in ('active', 'blob', 'ready', 'appdata'):
        (programs / name).mkdir(parents=True)
    (programs / 'operator.cap').write_bytes(op_cap)
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]

    def call(op, payload, ok=True):
        with closing(http.client.HTTPConnection('127.0.0.1', port, timeout=30)) as conn:
            conn.request('POST', '/', b'\x01' + token + bytes([1, op]) + payload)
            reply = conn.getresponse()
            result = reply.read()
        assert reply.status == 200 and len(result) >= 6, (op, reply.status)
        version, status, n = struct.unpack('<BBI', result[:6])
        assert version == 1 and n == len(result) - 6, (op, version, n)
        assert status == (0 if ok else 1), (op, status, result[6:])
        return result[6:]

    # Registration and staging store opaque code bytes; only invocation needs WASM.
    a = args.guest.read_bytes()
    b, c = b'recovery-stage-B', b'recovery-stage-C'
    ha, hb, hc = map(sha, (a, b, c))

    def active(h, rev):
        assert call(12, ident(iid)) == b'\x03\x01' + h + struct.pack('<Q', rev)

    def versions(*hashes):
        assert call(13, ident(iid) + b'\x00' + struct.pack('<H', 256)) == (
            struct.pack('<H', len(hashes)) + b''.join(sorted(hashes)))

    def stage(code, ok=True):
        return call(9, ident(admin) + blob(code), ok)

    def seal(h):
        assert call(10, ident(admin) + h) == b''

    def activate(rev, h, ok=True):
        return call(11, ident(admin) + struct.pack('<Q', rev) + h, ok)

    with server(args, base, port) as proc:
        call(15, ident(op_cap) + ident(admin) + blob(a))
        active(ha, 1)
        assert stage(b) == hb
        assert activate(1, hb, False) == b'\x08'
        versions(ha)
        proc.kill()
        proc.wait(timeout=5)
        assert proc.returncode == -9

    with server(args, base, port):
        active(ha, 1)
        versions(ha)
        assert (programs / 'blob' / (hb.hex() + '.bin')).read_bytes() == b'\x01' + b
        assert activate(1, hb, False) == b'\x08'
        assert stage(b) == hb
        seal(hb)
        activate(1, hb)
        active(hb, 2)
        versions(ha, hb)
        print('PASS: SIGKILL after stage, restart, idempotent stage, seal and activate', flush=True)

        # A directory at the exact blob target forces a real filesystem failure,
        # without consuming shared host storage or changing global permissions.
        blocked = programs / 'blob' / (hc.hex() + '.bin')
        blocked.mkdir()
        error = stage(c, False)
        assert error == b'\x07' and blocked.is_dir(), error
        active(hb, 2)
        versions(ha, hb)
        blocked.rmdir()
        assert stage(c) == hc
        seal(hc)
        activate(2, hc)
        active(hc, 3)
        versions(ha, hb, hc)
        print(f'PASS: filesystem obstruction (code {error.hex()}), recovery and no stuck lock', flush=True)

    with server(args, base, port):
        active(hc, 3)
        versions(ha, hb, hc)
        assert activate(1, ha, False)[:1] == b'\x0a'
        activate(3, ha)
        active(ha, 4)
        versions(ha, hb, hc)
        print('PASS: second restart, stale CAS rejection and rollback', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('kernel', 'p0', 'p1', 'guest'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    for name in ('kernel', 'p0', 'p1', 'guest'):
        print(name, sha(getattr(args, name).read_bytes()).hex(), flush=True)
    with tempfile.TemporaryDirectory(prefix='skrynia-registry-recovery-') as directory:
        run(args, Path(directory))
