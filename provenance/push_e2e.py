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

def put_u64(v: int) -> bytes:
    return struct.pack('<Q', v)

def push(op: int, *parts: bytes) -> bytes:
    """Ordinary request carrying one native P1 push-family opcode."""
    return b'\x01' + bytes([op]) + b''.join(parts)

def parse_hint_list(body: bytes):
    # body must be 4-byte count followed by 8-byte little-endian seqs
    if len(body) < 4 or (len(body) - 4) % 8 != 0:
        return None
    n = struct.unpack('<I', body[:4])[0]
    if 4 + 8 * n != len(body):
        return None
    return [struct.unpack('<Q', body[4 + 8 * i:12 + 8 * i])[0] for i in range(n)]

def parse_sub_fetch(body: bytes):
    if len(body) < 4:
        return None
    n = struct.unpack('<I', body[:4])[0]
    out, off = [], 4
    for _ in range(n):
        if off + 4 > len(body):
            return None
        ln = struct.unpack('<I', body[off:off + 4])[0]
        off += 4
        if off + ln > len(body):
            return None
        out.append(body[off:off + ln]); off += ln
    if off != len(body):
        return None
    return out

def run_push(work, kernel, p0, p1, echo_wasm, results):
    """Real kernel + dumb P0 + push-capable P1: native push opcodes over HTTP."""
    OP_GRANT, OP_SUB, OP_UNSUB, OP_SENDER = 18, 16, 17, 19
    OP_LIST, OP_READ, OP_ACK, OP_FETCH = 20, 21, 22, 23
    port = free_port()
    token = hashlib.sha256(b'dumb-p0-test-bearer-v1').digest()
    op_cap = hashlib.sha256(b'test-operator-cap-v1').digest()
    admin = hashlib.sha256(b'test-admin-cap-v1').digest()
    iid = hashlib.sha256(admin).digest()          # namespace read cap == ns for push
    base = work / 'push'
    srv = setup(base, port, kernel, p0, p1, token, op_cap=op_cap)
    echo = echo_wasm.read_bytes()
    key = b'notes/alpha'
    sibling = b'notes/beta'
    grant_cap = hashlib.sha256(b'test-grant-cap-v1').digest()
    bad_grant = hashlib.sha256(b'test-grant-cap-BAD').digest()
    sender_cap = hashlib.sha256(b'test-sender-cap-v1').digest()
    bad_sender = hashlib.sha256(b'test-sender-cap-BAD').digest()
    subscriber = b'sub-endpoint-alpha'
    subscription = b'https://127.0.0.1:9/push/A|p256dh=AAA|auth=BBB'
    try:
        # --- bootstrap ordinary P1 program ---
        d, _, _ = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(admin) + put_blob(echo), 'push-register')
        assert_ok(results, 'push-register', d)

        # --- POSITIVE: op18 grant, op16 subscribe, hint on write, op19-23 sender flow ---
        d, _, _ = srv.send(push(OP_GRANT, put_id(admin), put_id(grant_cap), put_blob(key), put_blob(subscriber)), 'op18-grant')
        assert_ok(results, 'op18-push-grant-positive', d)
        d, _, _ = srv.send(push(OP_SUB, put_id(grant_cap), put_blob(key), put_blob(subscriber), put_blob(subscription)), 'op16-sub')
        assert_ok(results, 'op16-push-subscribe-positive', d)
        d, _, _ = srv.send(b'\x01\x06' + put_id(admin) + put_blob(key) + b'\x00' + put_blob(b'v1'), 'put-hint')
        assert_ok(results, 'op07-app-put-generates-hint', d)
        d, _, _ = srv.send(push(OP_SENDER, put_id(admin), put_id(sender_cap), put_blob(key)), 'op19-sender-grant')
        assert_ok(results, 'op19-sender-grant-positive', d)
        d, _, _ = srv.send(push(OP_LIST, put_id(sender_cap), put_id(iid), put_blob(key)), 'op20-list')
        ok20 = d.get('kind') == 'ok'
        seqs = parse_hint_list(d.get('body', b'')) if ok20 else None
        rec(results, 'op20-hint-list-positive', ok20 and seqs is not None and len(seqs) >= 1,
            {'kind': d.get('kind'), 'seqs': seqs})
        seq = seqs[0] if seqs else 0
        d, _, _ = srv.send(push(OP_READ, put_id(sender_cap), put_id(iid), put_blob(key), put_u64(seq)), 'op21-read')
        h = hashlib.sha256(b'v1').digest()
        rec(results, 'op21-hint-read-positive', d.get('kind') == 'ok' and d.get('body') == h,
            {'kind': d.get('kind'), 'bytes': len(d.get('body', b'')), 'matches_new_hash': d.get('body') == h})
        d, _, _ = srv.send(push(OP_ACK, put_id(sender_cap), put_id(iid), put_blob(key), put_u64(seq)), 'op22-ack')
        assert_ok(results, 'op22-hint-ack-positive', d)
        d, _, _ = srv.send(push(OP_ACK, put_id(sender_cap), put_id(iid), put_blob(key), put_u64(seq)), 'op22-ack-replay')
        assert_ok(results, 'op22-hint-ack-idempotent-replay', d)
        d, _, _ = srv.send(push(OP_LIST, put_id(sender_cap), put_id(iid), put_blob(key)), 'op20-list-empty')
        rec(results, 'op20-hint-list-empty-after-ack', d.get('kind') == 'ok' and parse_hint_list(d.get('body', b'')) == [],
            {'kind': d.get('kind'), 'body': d.get('body')})
        d, _, _ = srv.send(push(OP_FETCH, put_id(sender_cap), put_id(iid), put_blob(key)), 'op23-fetch')
        subs = parse_sub_fetch(d.get('body', b'')) if d.get('kind') == 'ok' else None
        rec(results, 'op23-sub-fetch-scoped-positive', subs is not None and len(subs) >= 1 and subscription in subs,
            {'kind': d.get('kind'), 'subs': subs})

        # --- 256605-byte exact AppGet on the SAME new push-capable P1 binary ---
        large = bytes((i * 13) % 251 for i in range(256605))
        d, _, _ = srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'large') + b'\x00' + put_blob(large), 'push-large-put')
        assert_ok(results, 'push-large-app-put', d)
        d, _, _ = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'large'), 'push-large-get')
        rec(results, 'push-256605-appget-exact', d.get('kind') == 'ok' and d.get('body') == large,
            {'kind': d.get('kind'), 'bytes': len(d.get('body', b''))})

        # --- NEGATIVES on the real binary ---
        d, _, _ = srv.send(push(OP_SUB, put_id(bad_grant), put_blob(key), put_blob(subscriber), put_blob(subscription)), 'neg-wrong-grant')
        rec(results, 'neg-op16-wrong-grant-denied', d.get('kind') == 'err', d)
        d, _, _ = srv.send(push(OP_LIST, put_id(sender_cap), put_id(iid), put_blob(sibling)), 'neg-sibling-key')
        rec(results, 'neg-op20-sibling-key-denied', d.get('kind') == 'err', d)
        d, _, _ = srv.send(push(OP_LIST, put_id(bad_sender), put_id(iid), put_blob(key)), 'neg-wrong-sender')
        rec(results, 'neg-op20-wrong-sender-cap-denied', d.get('kind') == 'err', d)
        d, _, _ = srv.send(push(OP_FETCH, put_id(bad_sender), put_id(iid), put_blob(key)), 'neg-fetch-wrong-sender')
        rec(results, 'neg-op23-wrong-sender-cannot-read-endpoint', d.get('kind') == 'err', d)
        # unsupported opcode must fail closed
        d, _, _ = srv.send(b'\x01\x7f' + put_id(grant_cap), 'neg-unsupported-op')
        rec(results, 'neg-unsupported-opcode-fails-closed', d.get('kind') != 'ok', d)
        # malformed frame (truncated grant: op 18 with 2-byte cap, no rest) must fail closed
        d, _, _ = srv.send(b'\x01\x12' + b'\x02\x00\xaa\xbb', 'neg-malformed')
        rec(results, 'neg-malformed-frame-fails-closed', d.get('kind') != 'ok', d)
        # invalid object key grammar (wildcard) on grant must fail closed
        d, _, _ = srv.send(push(OP_GRANT, put_id(admin), put_id(grant_cap), put_blob(b'notes/*'), put_blob(subscriber)), 'neg-wildcard-key')
        rec(results, 'neg-wildcard-key-grant-denied', d.get('kind') == 'err', d)
        # --- revoke ABA: op17 unsubscribe then re-issue grant must not resurrect old endpoint ---
        d, _, _ = srv.send(push(OP_UNSUB, put_id(grant_cap), put_blob(key), put_blob(subscriber)), 'op17-unsub')
        assert_ok(results, 'op17-push-unsubscribe-positive', d)
        d, _, _ = srv.send(push(OP_FETCH, put_id(sender_cap), put_id(iid), put_blob(key)), 'fetch-after-unsub')
        rec(results, 'neg-revoke-removes-endpoint', d.get('kind') == 'ok' and parse_sub_fetch(d.get('body', b'')) == [],
            {'kind': d.get('kind'), 'body': d.get('body')})
        d, _, _ = srv.send(push(OP_GRANT, put_id(admin), put_id(grant_cap), put_blob(key), put_blob(subscriber)), 'op18-regrant')
        assert_ok(results, 'op18-reissue-identical-grant', d)
        d, _, _ = srv.send(push(OP_FETCH, put_id(sender_cap), put_id(iid), put_blob(key)), 'fetch-after-aba')
        rec(results, 'neg-revoke-aba-no-resurrection', d.get('kind') == 'ok' and parse_sub_fetch(d.get('body', b'')) == [],
            {'kind': d.get('kind'), 'body': d.get('body')})
        # --- subject-generation revocation: re-issue grant for a DIFFERENT subscriber ---
        subscriber2 = b'sub-endpoint-BETA'
        d, _, _ = srv.send(push(OP_GRANT, put_id(admin), put_id(grant_cap), put_blob(key), put_blob(subscriber2)), 'op18-rotate-subject')
        assert_ok(results, 'op18-rotate-grant-subject', d)
        d, _, _ = srv.send(push(OP_SUB, put_id(grant_cap), put_blob(key), put_blob(subscriber2), put_blob(subscription + b'-B')), 'op16-sub2')
        assert_ok(results, 'op16-subscribe-after-rotation', d)
        d, _, _ = srv.send(push(OP_FETCH, put_id(sender_cap), put_id(iid), put_blob(key)), 'fetch-after-rotation')
        subs2 = parse_sub_fetch(d.get('body', b'')) if d.get('kind') == 'ok' else None
        rec(results, 'neg-old-subject-revoked-by-generation', subs2 is not None and all((subscription + b'-B') == x for x in subs2),
            {'kind': d.get('kind'), 'subs': subs2})
    finally:
        try:
            srv.stop()
        except Exception:
            pass

    # --- real restart/persistence: reopen same dirs, re-drive sender flow ---
    env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(p0), 'SKRYNIA_OPERATOR_DIR': str(base / 'operator'),
           'SKRYNIA_USER_DIR': str(base / 'user'), 'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    log_path = base / 'server-push-restart.log'; log = open(log_path, 'wb')
    proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
    wait_port(port, proc, log_path)
    srv2 = Srv(base, port, token, proc, log)
    try:
        d, _, _ = srv2.send(b'\x01\x04' + put_id(iid), 'push-lookup-restart')
        assert_ok(results, 'push-lookup-after-restart', d)
        d, _, _ = srv2.send(push(OP_LIST, put_id(sender_cap), put_id(iid), put_blob(key)), 'list-restart')
        rec(results, 'push-sender-scope-survives-restart', d.get('kind') == 'ok', d)
        d, _, _ = srv2.send(push(OP_FETCH, put_id(sender_cap), put_id(iid), put_blob(key)), 'fetch-restart')
        rec(results, 'push-subscription-persists-restart', d.get('kind') == 'ok' and parse_sub_fetch(d.get('body', b'')) is not None,
            {'kind': d.get('kind')})
        d, _, _ = srv2.send(b'\x01\x07' + put_id(iid) + put_blob(b'large'), 'large-restart')
        rec(results, 'push-256605-appget-after-restart', d.get('kind') == 'ok' and d.get('body') == large,
            {'kind': d.get('kind'), 'bytes': len(d.get('body', b''))})
    finally:
        srv2.stop()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--kernel', required=True)
    ap.add_argument('--p0', required=True)
    ap.add_argument('--p1', required=True)
    ap.add_argument('--echo-guest', required=True)
    ap.add_argument('--push', action='store_true', help='run native push opcode matrix')
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
        if args.push:
            run_push(work, kernel, p0, p1, echo, results)
        else:
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
