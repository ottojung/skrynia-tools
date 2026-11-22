#!/usr/bin/env python3
"""Executable HTTP integration fixture for dumb Program0 (PR53).

Runs real compiled kernel + P0 + aa6333a P1 over HTTP. No mocks.
Bounded disposable temp dirs, fake raw 32-byte bearer, no real credentials.
"""
from __future__ import annotations
import argparse, hashlib, os, pathlib, shutil, socket, struct, subprocess, tempfile, time, sys

def put_id(b: bytes) -> bytes: return struct.pack('<H', len(b)) + b
def put_blob(b: bytes) -> bytes: return struct.pack('<I', len(b)) + b

def dechunk(r: bytes) -> bytes:
    out = bytearray(); i = 0
    while True:
        j = r.find(b'\r\n', i)
        if j < 0: return bytes(out) + r[i:]
        sz = int(r[i:j].split(b';', 1)[0], 16)
        if sz == 0: return bytes(out)
        out.extend(r[j + 2:j + 2 + sz]); i = j + 2 + sz + 2

def dec(r: bytes) -> dict:
    if len(r) < 6: return {'kind': 'short'}
    status = r[1]; blen = struct.unpack('<I', r[2:6])[0]; body = r[6:6 + blen]
    if status == 0: return {'kind': 'ok', 'body': body}
    if status == 1 and body: return {'kind': 'err', 'code': body[0]}
    return {'kind': 'other', 'status': status}

class Srv:
    def __init__(self, base, port, token, proc, log):
        self.base, self.port, self.token, self.proc, self.log = base, port, token, proc, log
    def stop(self):
        if self.proc.poll() is None:
            self.proc.terminate()
            try: self.proc.wait(timeout=5)
            except Exception: self.proc.kill(); self.proc.wait(timeout=5)
        self.log.close()
    def send(self, payload, label='', timeout=30, token=None):
        tok = self.token if token is None else token
        body = b'\x01' + tok + payload
        s = socket.create_connection(('127.0.0.1', self.port), 5); s.settimeout(timeout)
        hdr = (f'POST / HTTP/1.1\r\nHost: 127.0.0.1:{self.port}\r\n'
               f'Content-Length: {len(body)}\r\nConnection: close\r\n\r\n').encode()
        s.sendall(hdr + body); resp = b''
        while True:
            b = s.recv(65536)
            if not b: break
            resp += b
        s.close()
        h, r = resp.split(b'\r\n\r\n', 1)
        if b'transfer-encoding: chunked' in h.lower(): r = dechunk(r)
        d = dec(r); status = h.split(b'\r\n', 1)[0].decode()
        if label: print(f'{label}: {status} {d}', flush=True)
        return d, status, r

def sha256_file(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def check_sha(path: pathlib.Path, expected: str | None, name: str):
    if not expected: return
    got = sha256_file(path)
    if got != expected:
        raise SystemExit(f'{name} sha256 mismatch: expected {expected} got {got}')

def free_port() -> int:
    s = socket.socket(); s.bind(('127.0.0.1', 0))
    p = s.getsockname()[1]; s.close(); return p

def wait_port(port, proc, log_path, timeout=30):
    for _ in range(int(timeout / 0.2)):
        if proc.poll() is not None:
            err = log_path.read_text(errors='replace')[-500:]
            raise RuntimeError(f'server died rc={proc.returncode}\n{err}')
        try: socket.create_connection(('127.0.0.1', port), 0.2).close(); return
        except Exception: time.sleep(0.2)
    err = log_path.read_text(errors='replace')[-500:]
    raise RuntimeError('server did not listen\n' + err)

def setup(base, port, kernel, p0, p1, token, *, op_cap=None, shadow_verifier=None):
    if base.exists(): shutil.rmtree(base)
    base.mkdir(parents=True)
    op = base / 'operator'; us = base / 'user'
    (op / 'security').mkdir(parents=True); (op / 'programs').mkdir(parents=True)
    for s in ['active', 'blob', 'ready', 'appdata']:
        (us / 'programs' / s).mkdir(parents=True, exist_ok=True)
    (op / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(token).digest())
    if shadow_verifier is not None:
        (us / 'security').mkdir(parents=True, exist_ok=True)
        (us / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(shadow_verifier).digest())
    (op / 'programs' / 'root.wasm').write_bytes(p1.read_bytes())
    if op_cap: (us / 'programs' / 'operator.cap').write_bytes(op_cap)
    env = {**os.environ,
           'SKRYNIA_ROOT_WASM': str(p0),
           'SKRYNIA_OPERATOR_DIR': str(op),
           'SKRYNIA_USER_DIR': str(us),
           'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    log_path = base / 'server.log'
    log = open(log_path, 'wb')
    proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
    wait_port(port, proc, log_path)
    return Srv(base, port, token, proc, log)

def rec(results, name, ok, detail=''):
    results.append((name, bool(ok), str(detail)))
    print(('PASS' if ok else 'FAIL'), name, detail, flush=True)

def assert_ok(results, name, d, pred=lambda d: True):
    ok = d.get('kind') == 'ok' and pred(d)
    rec(results, name, ok, d); return ok

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--kernel', required=True)
    ap.add_argument('--p0', required=True)
    ap.add_argument('--p1', required=True)
    ap.add_argument('--echo-guest', required=True)
    ap.add_argument('--workdir')
    ap.add_argument('--kernel-sha256')
    ap.add_argument('--p0-sha256')
    ap.add_argument('--p1-sha256')
    ap.add_argument('--keep', action='store_true')
    args = ap.parse_args()
    kernel, p0, p1 = pathlib.Path(args.kernel), pathlib.Path(args.p0), pathlib.Path(args.p1)
    echo = pathlib.Path(args.echo_guest)
    for p, n in [(kernel,'kernel'),(p0,'p0'),(p1,'p1'),(echo,'echo-guest')]:
        if not p.is_file(): raise SystemExit(f'missing {n}: {p}')
    check_sha(kernel, args.kernel_sha256, 'kernel')
    check_sha(p0, args.p0_sha256, 'p0')
    check_sha(p1, args.p1_sha256, 'p1')
    work = pathlib.Path(args.workdir) if args.workdir else pathlib.Path(tempfile.mkdtemp(prefix='dumb-p0-http-'))
    work.mkdir(parents=True, exist_ok=True)
    results = []
    try:
        run(work, kernel, p0, p1, echo, results)
    finally:
        if not args.keep and not args.workdir:
            shutil.rmtree(work, ignore_errors=True)
    bad = [(n,d) for n,ok,d in results if not ok]
    print('ALL_OK=' + str(not bad))
    for n,d in bad: print('BAD', n, d)
    raise SystemExit(0 if not bad else 1)

def run(work, kernel, p0, p1, echo_wasm, results):
    port = free_port()
    # Fake raw test-only 32-byte bearer; never a real credential.
    token = hashlib.sha256(b'dumb-p0-test-bearer-v1').digest()
    op_cap = hashlib.sha256(b'test-operator-cap-v1').digest()
    admin = hashlib.sha256(b'test-admin-cap-v1').digest()
    iid = hashlib.sha256(admin).digest()
    base = work / 'pos'
    srv = setup(base, port, kernel, p0, p1, token, op_cap=op_cap)
    try:
        echo = echo_wasm.read_bytes()
        # first valid request in session
        d, _, _ = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(admin) + put_blob(echo), 'register')
        assert_ok(results, 'register', d)
        d, _, _ = srv.send(b'\x01\x04' + put_id(iid), 'lookup')
        assert_ok(results, 'lookup', d)
        d, _, _ = srv.send(b'\x01\x03' + put_id(iid) + put_blob(b'hello-dumb-p0'), 'invoke')
        assert_ok(results, 'invoke-p1-p2', d, lambda d: d.get('body') == b'hello-dumb-p0')
        # appdata write/read (requires 0xff write mode on 4389433)
        srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'k') + b'\x00' + put_blob(b'v1'), 'put1')
        d, _, _ = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get1')
        assert_ok(results, 'appdata-write-read', d, lambda d: d.get('body') == b'v1')
        # The real full-catalogue-sized object must read byte-exact under bounded VM fuel.
        large = bytes((i * 13) % 251 for i in range(256605))
        d, _, _ = srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'large') + b'\x00' + put_blob(large))
        assert_ok(results, 'appdata-large-put', d)
        d, _, _ = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'large'))
        rec(results, 'appdata-large-get-exact', d.get('kind') == 'ok' and d.get('body') == large,
            {'kind': d.get('kind'), 'bytes': len(d.get('body', b''))})
        # CAS
        h = hashlib.sha256(b'v1').digest()
        srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'k') + b'\x01' + h + put_blob(b'v2'), 'cas')
        d, _, _ = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get2')
        assert_ok(results, 'appdata-cas', d, lambda d: d.get('body') == b'v2')
        # wrong operator cap denied
        bad_admin = hashlib.sha256(b'bad-admin').digest()
        d, _, _ = srv.send(b'\x01\x0f' + put_id(hashlib.sha256(b'wrong-op').digest()) + put_id(bad_admin) + put_blob(echo), 'bad-op')
        rec(results, 'wrong-operator-cap-denied', d.get('kind') != 'ok', d)
        # sibling isolation: second program cannot read first program appdata by iid
        admin2 = hashlib.sha256(b'test-admin-cap-v2').digest(); iid2 = hashlib.sha256(admin2).digest()
        d, _, _ = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(admin2) + put_blob(echo), 'reg2')
        assert_ok(results, 'register-sibling', d)
        d, _, _ = srv.send(b'\x01\x07' + put_id(iid2) + put_blob(b'k'), 'get-sib')
        rec(results, 'sibling-appdata-isolated', d.get('kind') != 'ok' or d.get('body') != b'v2', d)
        srv.stop()
        # restart same dirs, same session expectations
        env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(p0), 'SKRYNIA_OPERATOR_DIR': str(base/'operator'),
               'SKRYNIA_USER_DIR': str(base/'user'), 'SKRYNIA_BIND': f'127.0.0.1:{port}'}
        log_path = base / 'server2.log'; log = open(log_path, 'wb')
        proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
        wait_port(port, proc, log_path)
        srv2 = Srv(base, port, token, proc, log)
        try:
            d, _, _ = srv2.send(b'\x01\x04' + put_id(iid), 'lookup-r')
            assert_ok(results, 'lookup-after-restart', d)
            d, _, _ = srv2.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get-r')
            assert_ok(results, 'appdata-after-restart', d, lambda d: d.get('body') == b'v2')
        finally:
            srv2.stop()
    finally:
        if srv.proc.poll() is None: srv.stop()

    run_negatives(work, kernel, p0, p1, results)

def run_negatives(work, kernel, p0, p1, results):
    port = free_port()
    token = hashlib.sha256(b'real-neg-bearer').digest()
    shadow = hashlib.sha256(b'shadow-user-verifier').digest()
    base = work / 'neg'
    srv = setup(base, port, kernel, p0, p1, token, shadow_verifier=shadow)
    try:
        iid = hashlib.sha256(b'x').digest()
        # wrong bearer
        d, status, _ = srv.send(b'\x01\x04' + put_id(iid), 'wrong', token=hashlib.sha256(b'wrong').digest())
        rec(results, 'wrong-bearer-denied', '500' in status or d.get('kind') != 'ok', status)
        # malformed/missing bearer
        body = b'\x01' + b'\x00' * 10 + b'\x01\x04' + put_id(iid)
        s = socket.create_connection(('127.0.0.1', port), 5); s.settimeout(10)
        hdr = (f'POST / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n').encode()
        s.sendall(hdr + body); resp = b''
        while True:
            b = s.recv(65536)
            if not b: break
            resp += b
        s.close()
        h, r = resp.split(b'\r\n\r\n', 1)
        status = h.split(b'\r\n', 1)[0].decode()
        if b'transfer-encoding: chunked' in h.lower(): r = dechunk(r)
        rec(results, 'missing-bearer-denied', '500' in status or dec(r).get('kind') != 'ok', status)
        # shadow verifier in user dir must not authorize
        d, status, _ = srv.send(b'\x01\x04' + put_id(iid), 'shadow', token=shadow)
        rec(results, 'shadow-user-verifier-denied', '500' in status or d.get('kind') != 'ok', status)
    finally:
        srv.stop()

if __name__ == '__main__':
    main()
