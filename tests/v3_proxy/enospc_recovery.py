#!/usr/bin/env python3
"""Genuine ENOSPC recovery of an ordinary nonzero-WASM persistent service.

Real-binary test module, not a filesystem mock. It drives a pinned generic
kernel -> token-gated Program0 -> ordinary nonzero P1 (cb-guest) and stores the
service's durable roots on a private, disposable small filesystem mounted by CI
(tmpfs, <=8 MiB). ENOSPC is produced by a bounded fixture that consumes the
free bytes of that exact volume; the failure is verified by a direct write
returning OSError errno 28 (ENOSPC). A directory at a blob target (EISDIR), an
RLIMIT-based fake, or a host-wide fill is never used as a substitute and never
counted as ENOSPC.

Phases: baseline commit B; stage a distinct C before filling; PRECOMMIT ENOSPC
(stage fails, no commit); POSTCOMMIT ENOSPC with the volume at zero free (real
SEAL(C) and ACTIVATE to an existing sealed release are attempted); free space,
restart the process with the SAME identity, and make a successful next
mutation. Fail-closed and uncertain-commit outcomes are both reconciled.

Run the real volume case only on a disposable GitHub runner with the compiled
stack artifact. Locally use --self-test (pure stdlib, no mounts, no fills).
"""
import argparse
from contextlib import closing
import errno
import hashlib
import http.client
import os
from pathlib import Path
import socket
import struct
import subprocess
import sys
import time

VERSION = 1
OP_REGISTER = 15
OP_STAGE = 9
OP_SEAL = 10
OP_ACTIVATE = 11
OP_GET_ACTIVE = 12
OP_LIST_VERSIONS = 13
ERR_STORE = b'\x07'
ERR_MISSING = b'\x08'
ERR_CONFLICT = b'\x0a\x00'  # conflict(None): code 10 plus a one-byte current-hash flag
PIN_KERNEL_SOURCE = 'db08cbc7496fafedc6a2e32df80395911b930879'
VOLUME_LIMIT_BYTES = 8 * 1024 * 1024
BALLAST_HEADROOM = 128 * 1024
STAGE_FAIL_BYTES = 1024 * 1024
PROBE_BYTES = 256 * 1024
PROBE_CHUNK = 4 * 1024
KERNEL_HEAD_KEY = 'kernel_head'
KERNEL_SOURCE_KEY = 'kernel'


def sha(value):
    return hashlib.sha256(value).digest()


def ident(value):
    return struct.pack('<H', len(value)) + value


def blob(value):
    return struct.pack('<I', len(value)) + value


def frame(token, op, payload):
    return b'\x01' + token + bytes([VERSION, op]) + payload


def parse_reply(raw):
    assert len(raw) >= 6, ('short reply', raw[:32])
    version, status, n = struct.unpack('<BBI', raw[:6])
    assert version == VERSION and n == len(raw) - 6, (version, n, len(raw))
    return status, raw[6:]


def volume_info(volume):
    st = os.statvfs(volume)
    fstype = 'unknown'
    options = ''
    for line in Path('/proc/mounts').read_text().splitlines():
        parts = line.split()
        if len(parts) >= 4 and os.path.realpath(parts[1]) == os.path.realpath(volume):
            fstype, options = parts[2], parts[3]
            break
    return {
        'path': str(volume),
        'fstype': fstype,
        'options': options,
        'total': st.f_blocks * st.f_frsize,
        'free': st.f_bavail * st.f_frsize,
    }


def describe_volume(volume, label):
    info = volume_info(volume)
    print(f'{label}: {info}', flush=True)
    assert info['total'] <= VOLUME_LIMIT_BYTES, info
    assert info['fstype'] == 'tmpfs', info
    return info


def fill_ballast(volume, headroom=BALLAST_HEADROOM):
    """Consume almost all free bytes, or all of them when headroom is zero."""
    before = volume_info(volume)
    target = before['free'] - headroom
    assert target >= 2 * STAGE_FAIL_BYTES, (before, target)
    path = volume / 'enospc-ballast.bin'
    assert not path.exists(), path
    chunk = b'\xA5' * (64 * 1024)
    written = 0
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        while written < target:
            amount = min(len(chunk), target - written)
            try:
                written += os.write(fd, chunk[:amount])
            except OSError as error:
                assert headroom == 0 and error.errno == errno.ENOSPC, (headroom, error)
                break
    finally:
        os.close(fd)
    after = volume_info(volume)
    if headroom == 0:
        assert after['free'] == 0, after
    else:
        assert 0 < after['free'] <= headroom * 2, (before, after)
    print(f'BALLAST: wrote {written} bytes (headroom {headroom}); '
          f'free {before["free"]} -> {after["free"]}', flush=True)
    return path


def probe_enospc(volume):
    """Prove the exact volume is genuinely full with a direct OSError errno."""
    path = volume / 'enospc-probe.bin'
    chunk = b'\x5a' * PROBE_CHUNK
    written = 0
    caught = None
    fd = None
    try:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        while written < PROBE_BYTES:
            written += os.write(fd, chunk)
    except OSError as error:
        caught = error
    finally:
        if fd is not None:
            os.close(fd)
        if path.exists():
            path.unlink()
    assert caught is not None, ('probe succeeded; volume was not full', volume_info(volume))
    assert caught.errno == errno.ENOSPC, (caught, errno.errorcode.get(caught.errno))
    print(f'ENOSPC FIXTURE: direct write to {path} -> OSError errno={caught.errno} '
          f'({errno.errorcode[caught.errno]}): {caught}; bytes accepted={written}; '
          f'volume={volume_info(volume)}', flush=True)
    return caught.errno


def free_space(ballast):
    for path in (ballast, ballast.parent / 'enospc-probe.bin'):
        if path.exists():
            path.unlink()
    print(f'FREED: ballast/probe removed; {volume_info(ballast.parent)}', flush=True)


def pick_port():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        return sock.getsockname()[1]


def start_server(args, base, port, log_dir):
    env = {**os.environ,
           'SKRYNIA_ROOT_WASM': str(args.p0.resolve()),
           'SKRYNIA_OPERATOR_DIR': str(base / 'operator'),
           'SKRYNIA_USER_DIR': str(base / 'user'),
           'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    env.pop('SKRYNIA_SYSCALL_URL', None)
    log = (log_dir / 'kernel.log').open('ab')
    proc = subprocess.Popen([str(args.kernel.resolve())], env=env,
                            stdout=log, stderr=subprocess.STDOUT)
    for _ in range(200):
        assert proc.poll() is None, ('kernel exited', log.name)
        try:
            with socket.create_connection(('127.0.0.1', port), 0.2):
                return proc, log
        except OSError:
            time.sleep(0.2)
    proc.kill()
    proc.wait(timeout=5)
    raise RuntimeError('fixture kernel did not listen')


def stop_server(proc, log):
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)
    log.close()


class Fixture:
    def __init__(self, args, base, port, log_dir, identity=None):
        self.args = args
        self.base = base
        self.port = port
        self.log_dir = log_dir
        if identity is None:
            self.token, self.op_cap, self.admin = (os.urandom(32) for _ in range(3))
        else:
            self.token, self.op_cap, self.admin = identity
        self.iid = sha(self.admin)
        self.proc = None
        self.log = None

    def credentials(self):
        """Opaque identity, reused across a process restart; never logged."""
        return (self.token, self.op_cap, self.admin)

    def __enter__(self):
        operator = self.base / 'operator'
        (operator / 'security').mkdir(parents=True, exist_ok=True)
        (operator / 'programs').mkdir(parents=True, exist_ok=True)
        (operator / 'security/program0.sha256').write_bytes(sha(self.token))
        (operator / 'programs/root.wasm').write_bytes(self.args.p1.read_bytes())
        programs = self.base / 'user' / 'programs'
        for name in ('active', 'blob', 'ready', 'appdata'):
            (programs / name).mkdir(parents=True, exist_ok=True)
        (programs / 'operator.cap').write_bytes(self.op_cap)
        self.proc, self.log = start_server(self.args, self.base, self.port, self.log_dir)
        return self

    def __exit__(self, *_exc):
        stop_server(self.proc, self.log)

    def call_status(self, op, payload):
        with closing(http.client.HTTPConnection('127.0.0.1', self.port, timeout=90)) as conn:
            conn.request('POST', '/', frame(self.token, op, payload))
            reply = conn.getresponse()
            raw = reply.read()
        assert reply.status == 200, (op, reply.status, raw[:200])
        return parse_reply(raw)

    def call(self, op, payload, ok=True):
        status, body = self.call_status(op, payload)
        assert status == (0 if ok else 1), (op, status, body[:64])
        return body

    def active(self, code_hash, revision):
        expected = b'\x03\x01' + code_hash + struct.pack('<Q', revision)
        assert self.call(OP_GET_ACTIVE, ident(self.iid)) == expected, (code_hash, revision)

    def versions(self, *hashes):
        # A page size of 256 is accepted by the current protocol; a future
        # max64 revision is deferred to charging coordination (64 already
        # works on the old protocol). No page-size change is made here.
        body = self.call(OP_LIST_VERSIONS, ident(self.iid) + b'\x00' + struct.pack('<H', 64))
        expected = struct.pack('<H', len(hashes)) + b''.join(sorted(hashes))
        assert body == expected, (body.hex(), expected.hex())

    def register(self, code):
        self.call(OP_REGISTER, ident(self.op_cap) + ident(self.admin) + blob(code))

    def stage(self, code, ok=True):
        return self.call(OP_STAGE, ident(self.admin) + blob(code), ok)

    def seal(self, code_hash, ok=True):
        return self.call(OP_SEAL, ident(self.admin) + code_hash, ok)

    def activate(self, revision, code_hash, ok=True):
        return self.call(OP_ACTIVATE,
                         ident(self.admin) + struct.pack('<Q', revision) + code_hash, ok)


def check_pins(args):
    manifest = {}
    for line in args.manifest.read_text().splitlines():
        key, separator, value = line.partition('=')
        if separator:
            manifest[key.strip()] = value.strip()
    paths = {
        'kernel': args.kernel,
        'p0': args.p0,
        'p1': args.p1,
        'p2': args.guest,
    }
    digests = {}
    for name, path in paths.items():
        digests[name] = sha(path.read_bytes()).hex()
        wanted = manifest.get(f'sha256_{name}')
        assert digests[name] == wanted, (name, digests[name], wanted)
    assert manifest.get(KERNEL_SOURCE_KEY) == PIN_KERNEL_SOURCE, manifest
    assert manifest.get(KERNEL_HEAD_KEY) == PIN_KERNEL_SOURCE, manifest
    assert manifest.get('tools_head'), manifest
    print(f'PINS: kernel_source={PIN_KERNEL_SOURCE} tools_head={manifest["tools_head"]} '
          + ' '.join(f'sha256_{name}={digests[name][:12]}' for name in paths), flush=True)


def run(args):
    check_pins(args)
    volume = args.volume.resolve()
    assert volume.is_dir(), volume
    describe_volume(volume, 'VOLUME(before)')
    assert args.log_dir.is_dir(), args.log_dir
    base = volume / 'data'
    base.mkdir(parents=True, exist_ok=True)
    port = pick_port()
    log_dir = args.log_dir.resolve()
    guest = args.guest.read_bytes()
    hash_a, hash_b, hash_c = sha(guest), sha(b'ordinary-service-B'), sha(b'ordinary-service-C')
    identity = None

    with Fixture(args, base, port, log_dir, identity) as fx:
        identity = fx.credentials()
        fx.register(guest)
        fx.seal(hash_a)
        fx.active(hash_a, 1)
        fx.versions(hash_a)
        assert fx.stage(b'ordinary-service-B') == hash_b
        fx.seal(hash_b)
        fx.activate(1, hash_b)
        fx.active(hash_b, 2)
        fx.versions(hash_a, hash_b)
        assert fx.stage(b'ordinary-service-C') == hash_c
        assert fx.activate(1, hash_a, ok=False) == ERR_CONFLICT, 'stale CAS must fail closed'
        print('PASS: baseline A rev=1, committed B rev=2, distinct C staged pre-fill',
              'stale CAS rejected', flush=True)

        # PRECOMMIT ENOSPC: almost-full volume; stage fails before any commit.
        ballast = fill_ballast(volume)
        failed_hash = sha(b'P' * STAGE_FAIL_BYTES)
        assert fx.stage(b'P' * STAGE_FAIL_BYTES, ok=False) == ERR_STORE, 'stage store error'
        eno = probe_enospc(volume)
        assert eno == errno.ENOSPC
        fx.active(hash_b, 2)
        fx.versions(hash_a, hash_b)
        assert fx.activate(2, failed_hash, ok=False) == ERR_MISSING, 'unknown fail closed'
        print(f'PASS: PRECOMMIT ENOSPC errno={eno}: stage rejected, active B rev=2 '
              'and history preserved, unknown rejected', flush=True)
        free_space(ballast)

        # POSTCOMMIT ENOSPC: volume at zero free, real SEAL and ACTIVATE paths.
        ballast = fill_ballast(volume, headroom=0)
        eno = probe_enospc(volume)
        assert eno == errno.ENOSPC
        status, body = fx.call_status(OP_SEAL, ident(fx.admin) + hash_c)
        assert status == 1 and body == ERR_STORE, ('seal under zero free must fail', status, body)
        fx.active(hash_b, 2)
        fx.versions(hash_a, hash_b)
        status, body = fx.call_status(
            OP_ACTIVATE, ident(fx.admin) + struct.pack('<Q', 2) + hash_a)
        if status == 1:
            assert body == ERR_STORE, ('activate under zero free error', status, body)
            outcome = 'fail-closed'
            fx.active(hash_b, 2)
            fx.versions(hash_a, hash_b)
        else:
            assert body == b'\x03\x01' + hash_a + struct.pack('<Q', 3), (status, body)
            outcome = 'committed-under-zero-free'
            fx.active(hash_a, 3)
            fx.versions(hash_a, hash_b)
        print(f'PASS: POSTCOMMIT zero-free ENOSPC errno={eno}: SEAL(C) rejected; '
              f'ACTIVATE {outcome}; active/history consistent', flush=True)
        free_space(ballast)

    # Restart the process only; the SAME token/op_cap/admin and iid are reused.
    with Fixture(args, base, port, log_dir, identity) as fx:
        if outcome == 'fail-closed':
            fx.active(hash_b, 2)
            fx.versions(hash_a, hash_b)
            fx.activate(2, hash_a)
            fx.active(hash_a, 3)
            fx.versions(hash_a, hash_b)
            revision = 3
            next_name = 'rollback to sealed A'
        else:
            fx.active(hash_a, 3)
            fx.versions(hash_a, hash_b)
            fx.activate(3, hash_b)
            fx.active(hash_b, 4)
            fx.versions(hash_a, hash_b)
            revision = 4
            next_name = 'forward activation of B'
        assert fx.activate(revision, sha(b'never-staged'), ok=False) == ERR_MISSING
        assert fx.activate(revision - 1, hash_b if revision == 3 else hash_a,
                           ok=False) == ERR_CONFLICT, 'stale CAS must fail closed'
        print(f'PASS: RESTART with same identity: {next_name} succeeded '
              f'(rev={revision}), lock recovered, unknown fail closed', flush=True)

    describe_volume(volume, 'VOLUME(after)')
    print('ALL PASS: genuine ENOSPC precommit/postcommit, fail-closed, recovery', flush=True)


def self_test():
    """Pure stdlib local unit test: no mount, no fills, no kernel required."""
    assert errno.ENOSPC == 28
    token = b't' * 32
    assert ident(b'ab') == b'\x02\x00ab'
    assert blob(b'abcd') == struct.pack('<I', 4) + b'abcd'
    request = frame(token, OP_STAGE, blob(b'x'))
    assert request[:1] == b'\x01' and request[1:33] == token
    assert request[33:35] == bytes([VERSION, OP_STAGE])
    assert parse_reply(b'\x01\x00' + struct.pack('<I', 1) + ERR_STORE) == (0, ERR_STORE)
    assert parse_reply(b'\x01\x01' + struct.pack('<I', 1) + ERR_MISSING) == (1, ERR_MISSING)
    try:
        parse_reply(b'\x01\x00' + struct.pack('<I', 2) + b'\x07')
    except AssertionError:
        pass
    else:
        raise AssertionError('mismatched reply length accepted')
    source = Path(__file__).read_text()
    # Built by concatenation so the guard itself cannot trigger the guard.
    forbidden = ('blocked' + '.mkdir', 'RLIMIT_' + 'FSIZE')
    for token in forbidden:
        assert token not in source, f'{token}: EISDIR/rlimit substitutes must not be copied here'
    print('SELF-TEST PASS: framing, error codes, no EISDIR/rlimit substitutes', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--kernel', type=Path)
    parser.add_argument('--p0', type=Path)
    parser.add_argument('--p1', type=Path)
    parser.add_argument('--guest', type=Path)
    parser.add_argument('--volume', type=Path)
    parser.add_argument('--log-dir', type=Path)
    parser.add_argument('--manifest', type=Path)
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    required = ('kernel', 'p0', 'p1', 'guest', 'volume', 'log_dir', 'manifest')
    missing = [name for name in required if getattr(args, name) is None]
    assert not missing, f'missing required options: {missing} (or use --self-test)'
    run(args)
    return 0


if __name__ == '__main__':
    sys.exit(main())
