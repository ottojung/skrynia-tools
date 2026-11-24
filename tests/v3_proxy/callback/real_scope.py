#!/usr/bin/env python3
"""Disposable compiled P0 callback scope checks, using only synthetic data."""
import argparse
from contextlib import closing
import hashlib
import http.client
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time


def u32(n):
    return struct.pack('<I', n)


def path(value):
    return u32(len(value)) + value


def request(port, token, payload):
    with closing(http.client.HTTPConnection('127.0.0.1', port, timeout=20)) as conn:
        conn.request('POST', '/', b'\x01' + token + payload)
        reply = conn.getresponse()
        return reply.status, reply.read()


def run(args):
    token = hashlib.sha256(b'disposable callback scope fixture').digest()
    with tempfile.TemporaryDirectory(prefix='skrynia-scope-') as directory:
        base = Path(directory)
        operator, user = base / 'operator', base / 'user'
        (operator / 'security').mkdir(parents=True)
        (operator / 'programs').mkdir()
        (user / 'data').mkdir(parents=True)
        verifier = hashlib.sha256(token).digest()
        guest = args.guest.read_bytes()
        (operator / 'security/program0.sha256').write_bytes(verifier)
        (operator / 'programs/root.wasm').write_bytes(guest)
        (user / 'data/allowed').write_bytes(b'allowed user data')
        (user / 'escape').symlink_to(operator / 'security/program0.sha256')
        (user / 'escape-dir').symlink_to(operator / 'security', target_is_directory=True)
        with socket.socket() as s:
            s.bind(('127.0.0.1', 0))
            port = s.getsockname()[1]
        env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(args.p0.resolve()),
               'SKRYNIA_OPERATOR_DIR': str(operator), 'SKRYNIA_USER_DIR': str(user),
               'SKRYNIA_BIND': f'127.0.0.1:{port}'}
        # Do not inherit a real external syscall provider configuration.
        env.pop('SKRYNIA_SYSCALL_URL', None)
        with (base / 'server.log').open('wb') as log:
            proc = subprocess.Popen([str(args.kernel.resolve())], env=env, stdout=log, stderr=subprocess.STDOUT)
            try:
                for _ in range(150):
                    if proc.poll() is not None:
                        raise RuntimeError('fixture kernel exited')
                    try:
                        with socket.create_connection(('127.0.0.1', port), 0.2):
                            break
                    except OSError:
                        time.sleep(0.2)
                else:
                    raise RuntimeError('fixture kernel did not listen')
                checks(port, token, operator, user, verifier, guest)
            finally:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)


def checks(port, token, operator, user, verifier, guest):
    def call(label, op, value, tail=b''):
        status, reply = request(port, token, bytes([op]) + path(value) + tail)
        assert status == 200 and len(reply) >= 4, (label, status, len(reply))
        code = struct.unpack('<i', reply[:4])[0]
        print(label, code, flush=True)
        return code, reply[4:]

    def allowed():
        assert call('user-positive', 1, b'data/allowed', u32(1)) == (17, b'allowed user data')

    def denied(label, op, value, tail=b''):
        code, reply = call(label, op, value, tail)
        assert code < 0 and not reply, (label, code, len(reply))

    allowed()
    for value in (b'security/program0.sha256', b'programs/root.wasm',
                  b'../operator/security/program0.sha256', b'/security/program0.sha256',
                  b'escape', b'escape-dir/program0.sha256'):
        denied('read-denied:' + value.decode(), 1, value, u32(1))
    # Delete is idempotent for an absent user-root path. Confirm the
    # operator bytes remain intact, not an assumed error code.
    assert call('delete-absent-user-path', 6, b'security/program0.sha256') == (0, b'')
    denied('operator-rename', 7, b'programs/root.wasm', path(b'data/stolen'))
    denied('operator-directory', 8, b'security')
    denied('traversal-rename', 7, b'data/allowed', path(b'../operator/programs/root.wasm'))
    denied('symlink-directory-rename', 7, b'data/allowed', path(b'escape-dir/program0.sha256'))
    status, reply = request(port, token, bytes([15]))
    assert status == 200 and reply == struct.pack('<i', -4), ('callback-eval', status)
    print('callback-eval denied', flush=True)
    assert (operator / 'security/program0.sha256').read_bytes() == verifier
    assert (operator / 'programs/root.wasm').read_bytes() == guest
    assert not (user / 'data/stolen').exists()
    # A user-root twin of an operator path is ordinary user data.
    (user / 'security').mkdir()
    (user / 'security/program0.sha256').write_bytes(b'user twin')
    assert call('user-twin-read', 1, b'security/program0.sha256', u32(1)) == (9, b'user twin')
    assert call('user-twin-delete', 6, b'security/program0.sha256') == (0, b'')
    assert not (user / 'security/program0.sha256').exists()
    assert (operator / 'security/program0.sha256').read_bytes() == verifier
    allowed()
    status, _ = request(port, bytes(32), b'\x01' + path(b'data/allowed') + u32(1))
    assert status != 200, ('wrong bearer', status)
    status, _ = request(port, token, b'\xff' + path(b'data/allowed') + u32(1))
    assert status != 200, ('trap', status)
    allowed()
    print('PASS: compiled P0 callback isolates operator root, including after a trap', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('kernel', 'p0', 'guest'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    for name in ('kernel', 'p0', 'guest'):
        artifact = getattr(args, name)
        print(name, hashlib.sha256(artifact.read_bytes()).hexdigest(), flush=True)
    run(args)
