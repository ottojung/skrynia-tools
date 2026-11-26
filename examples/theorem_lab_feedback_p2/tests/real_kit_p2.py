#!/usr/bin/env python3
"""Real protocol fixture: compiled Theorem Lab P2 over the fixed generic kernel + PR53 P0 + clean P1.

Runs the real compiled binaries over HTTP in disposable dirs. No mocks, no real
credentials (fake 32-byte bearer only). Verifies mediated self-namespace CAS,
restart persistence, sibling isolation, unauth invoke deny and write-cap forging.
"""
from __future__ import annotations
import argparse, hashlib, os, pathlib, shutil, socket, struct, subprocess, tempfile, time, json
from assertions import forge_denied, sibling_no_leak

KEY = b'theorem-lab/feedback-v1'
# Exact target-stack kit pins, re-derived from bytes in the decisive compiled
# run (see docs/compiled-stack-final-pins-KlndxFPa91ne10cVuNvNQV-20261126.md).
# The kernel is a fresh rebuild of K10 db08cbc with explicit root namespaces;
# P0/P1 are the scoped T10 artifacts, and P2 is the A3 real-guest artifact that
# carries a live P1-owned callback context. The old kit pins are retired: the
# earlier P1 silently returned empty invoke bodies with the P2 example guest.
# CI supplies its own freshly built digests with --*-sha256; a bare run fails
# closed unless it is given this exact tested set.
SHA = {
    'kernel': '3da1852e1294f090bcee2c050f12e961e844ae5ae2d144b2453fd3cd2d54e89b',
    'p0': '3f2d40321c2d2af8a77cbc210f863c963492f1b021885a88d963e8b5bd957357',
    'p1': 'ec84e056a209d4d58d990a231070c1b55f21f7b564a676426fdcb3599d94fc23',
    'p2': '19a592588d9f0987ec93b55837778aadf9673c6b35e814cacfa2f1d043e8d11a',
}

def put_id(b): return struct.pack('<H', len(b)) + b
def put_blob(b): return struct.pack('<I', len(b)) + b

def dechunk(r):
    out = bytearray(); i = 0
    while True:
        j = r.find(b'\r\n', i)
        if j < 0: return bytes(out) + r[i:]
        sz = int(r[i:j].split(b';', 1)[0], 16)
        if sz == 0: return bytes(out)
        out.extend(r[j + 2:j + 2 + sz]); i = j + 2 + sz + 2

def dec(r):
    if len(r) < 6: return {'kind': 'short', 'hex': r.hex()}
    status = r[1]; blen = struct.unpack('<I', r[2:6])[0]; body = r[6:6 + blen]
    if status == 0: return {'kind': 'ok', 'body': body}
    if status == 1 and body: return {'kind': 'err', 'code': body[0]}
    return {'kind': 'other', 'status': status}

def free_port():
    s = socket.socket(); s.bind(('127.0.0.1', 0)); p = s.getsockname()[1]; s.close(); return p

def wait_port(port, proc, log_path, timeout=30):
    for _ in range(int(timeout / 0.2)):
        if proc.poll() is not None:
            raise RuntimeError(f'server died rc={proc.returncode}\n{log_path.read_text(errors="replace")[-800:]}')
        try: socket.create_connection(('127.0.0.1', port), 0.2).close(); return
        except Exception: time.sleep(0.2)
    raise RuntimeError('server did not listen\n' + log_path.read_text(errors='replace')[-800:])

class Srv:
    def __init__(self, base, port, token, proc, log):
        self.base, self.port, self.token, self.proc, self.log = base, port, token, proc, log
    def stop(self):
        if self.proc.poll() is None:
            self.proc.terminate()
            try: self.proc.wait(timeout=5)
            except Exception: self.proc.kill(); self.proc.wait(timeout=5)
        self.log.close()
    def send(self, payload, label='', token=None, timeout=30):
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
        d = dec(r)
        if label: print(f'{label}: {h.splitlines()[0].decode()} {d}', flush=True)
        return d

def start(base, port, kernel, p0, p1, token, op_cap):
    if base.exists(): shutil.rmtree(base)
    base.mkdir(parents=True)
    op, us = base / 'operator', base / 'user'
    (op / 'security').mkdir(parents=True); (op / 'programs').mkdir(parents=True)
    for s in ['active', 'blob', 'ready', 'appdata']:
        (us / 'programs' / s).mkdir(parents=True, exist_ok=True)
    (op / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(token).digest())
    (op / 'programs' / 'root.wasm').write_bytes(p1.read_bytes())
    if op_cap: (us / 'programs' / 'operator.cap').write_bytes(op_cap)
    return _launch(base, port, kernel, p0, token)

def _launch(base, port, kernel, p0, token):
    env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(p0), 'SKRYNIA_OPERATOR_DIR': str(base / 'operator'),
           'SKRYNIA_USER_DIR': str(base / 'user'), 'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    log_path = base / ('server.log' if not (base / 'server.log').exists() else 'server2.log')
    log = open(log_path, 'wb')
    proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
    wait_port(port, proc, log_path)
    return Srv(base, port, token, proc, log)

def submit_payload(iid, eid):
    return json.dumps({'v': 1, 'op': 1, 'invokeId': iid.hex(),
        'entry': {'id': eid, 'comment': 'secure feedback', 'pageUrl': 'https://example.test/t',
                  'htmlChain': []},
        'fragments': {}, 'updatedAt': '2026-11-21T00:00:00Z'}).encode()

def run(work, kernel, p0, p1, p2, results):
    token = hashlib.sha256(b'theorem-p2-test-bearer').digest()
    op_cap = hashlib.sha256(b'theorem-p2-operator-cap').digest()
    admin = hashlib.sha256(b'theorem-p2-admin-cap').digest()
    iid = hashlib.sha256(admin).digest()
    base = work / 'pos'; port = free_port()
    srv = start(base, port, kernel, p0, p1, token, op_cap)
    try:
        code = p2.read_bytes()
        d = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(admin) + put_blob(code), 'register-p2')
        rec(results, 'register-p2', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(submit_payload(iid, 'e1')), 'submit-e1')
        v = parse_json(results, 'submit-e1', d)
        rec(results, 'submit-e1-created', bool(v and v.get('ok') and v.get('created')), d)
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(KEY), 'get-state-1')
        st = parse_json(results, 'get-state-1', d)
        rec(results, 'state-has-e1', bool(st and [e['id'] for e in st.get('feedback', [])] == ['e1']), d)
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(submit_payload(iid, 'e2')), 'submit-e2')
        v = parse_json(results, 'submit-e2', d)
        rec(results, 'submit-e2-created', bool(v and v.get('ok') and v.get('created')), d)
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(KEY), 'get-state-2')
        st = parse_json(results, 'get-state-2', d)
        rec(results, 'state-has-e1-e2', bool(st and [e['id'] for e in st.get('feedback', [])] == ['e1', 'e2']), d)
        final_count = concurrent_appends(srv, iid, results)
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(submit_payload(iid, 'e1')), 'submit-e1-again')
        v = parse_json(results, 'submit-e1-again', d)
        rec(results, 'idempotent-not-created', bool(v and v.get('ok') and not v.get('created')), d)
        # Write-cap forging: the forged write to 'forge-key' must be denied so
        # the key is not readable. The old check re-read KEY and compared only
        # its feedback count, which passes even when the forge succeeds.
        forged = hashlib.sha256(b'evil').digest()
        srv.send(b'\x01\x06' + put_id(iid) + put_blob(b'forge-key') + b'\x00' + put_blob(forged), 'forge-put')
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'forge-key'), 'get-forge-key')
        rec(results, 'write-cap-forge-denied', forge_denied(d), d)
        # Secondary invariant: the forged write must not disturb real state.
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(KEY), 'get-state-forged')
        st = parse_json(results, 'get-state-forged', d)
        rec(results, 'forged-write-left-state-intact', bool(st and len(st.get('feedback', [])) == final_count), d)
        srv.stop()
        srv2 = _launch(base, port, kernel, p0, token)
        try:
            d = srv2.send(b'\x01\x04' + put_id(iid), 'lookup-restart')
            rec(results, 'lookup-after-restart', d.get('kind') == 'ok', d)
            d = srv2.send(b'\x01\x07' + put_id(iid) + put_blob(KEY), 'get-state-restart')
            st = parse_json(results, 'get-state-restart', d)
            rec(results, 'state-after-restart', bool(st and len(st.get('feedback', [])) == final_count), d)
        finally:
            srv2.stop()
    finally:
        if srv.proc.poll() is None: srv.stop()
    run_denials(work, kernel, p0, p1, p2, token, op_cap, results)

def run_denials(work, kernel, p0, p1, p2, token, op_cap, results):
    port = free_port(); admin = hashlib.sha256(b'theorem-p2-admin-cap').digest()
    iid = hashlib.sha256(admin).digest()
    sibling = hashlib.sha256(b'theorem-p2-sibling-namespace').digest()
    base = work / 'neg'; srv = start(base, port, kernel, p0, p1, token, op_cap)
    try:
        d = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(admin) + put_blob(p2.read_bytes()), 'reg-victim')
        rec(results, 'reg-victim', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(submit_payload(iid, 'secret')), 'victim-write')
        v = parse_json(results, 'victim-write', d)
        rec(results, 'victim-write-ok', bool(v and v.get('ok')), d)
        # Sibling namespace isolation: an unregistered sibling read cap must
        # get a denial. The old check compared the body to b'secret' although
        # the stored value is a JSON state blob containing "id":"secret", so a
        # real leak of that blob passed unnoticed.
        d = srv.send(b'\x01\x07' + put_id(sibling) + put_blob(KEY), 'sibling-read')
        rec(results, 'sibling-namespace-isolated', sibling_no_leak(d), d)
        # unauth invoke: invoke real P2 but claim a foreign invokeId in JSON.
        # P2 uses that value as its AppGet/AppPut cap, the P1 callback requires
        # cap == the actual invoke_id, so the mediated write is denied.
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(submit_payload(sibling, 'wrong')), 'wrong-iid-invoke')
        v = parse_json(results, 'wrong-iid-invoke', d)
        rec(results, 'unauth-invoke-denied', bool(v and v.get('ok') is False), d)
        # victim namespace must be untouched by the denied write
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(KEY), 'victim-state-after-deny')
        st = parse_json(results, 'victim-state-after-deny', d)
        rec(results, 'victim-state-intact', bool(st and [e['id'] for e in st.get('feedback', [])] == ['secret']), d)
    finally:
        if srv.proc.poll() is None: srv.stop()

def concurrent_appends(srv, iid, results):
    # Two simultaneous submits contend on the same self-namespace state.
    # Safe outcomes: ok, or a structured bounded "busy" (caller may retry).
    # The invariant is no lost/corrupt update: state equals exactly the
    # successfully committed entries.
    from concurrent.futures import ThreadPoolExecutor
    def one(eid):
        return srv.send(b'\x01\x03' + put_id(iid) + put_blob(submit_payload(iid, eid)), f'conc-{eid}')
    with ThreadPoolExecutor(max_workers=2) as pool:
        replies = list(pool.map(one, ['e3', 'e4']))
    committed = []
    for reply, eid in zip(replies, ['e3', 'e4']):
        v = parse_json(results, f'conc-{eid}', reply)
        if v and v.get('ok'):
            committed.append(eid)
        else:
            rec(results, f'concurrent-{eid}-structured-busy',
                bool(v and v.get('ok') is False and v.get('error') == 'busy'), reply)
    rec(results, 'concurrent-at-least-one-committed', bool(committed), replies)
    d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(KEY), 'get-state-concurrent')
    st = parse_json(results, 'get-state-concurrent', d)
    ids = sorted(e['id'] for e in st.get('feedback', [])) if st else []
    expected = sorted(['e1', 'e2'] + committed)
    rec(results, 'concurrent-no-lost-update', ids == expected, d)
    return len(expected)

def parse_json(results, name, d):
    try: return json.loads(d.get('body', b''))
    except Exception: rec(results, name + '-json', False, d); return None

def rec(results, name, ok, detail=''):
    results.append((name, bool(ok), str(detail)))
    print(('PASS' if ok else 'FAIL'), name, detail, flush=True)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--kernel', required=True); ap.add_argument('--p0', required=True)
    ap.add_argument('--p1', required=True); ap.add_argument('--p2', required=True)
    # CI supplies digests computed by the trusted build step so the freshly
    # compiled artifacts are still pin-verified from bytes at startup.
    for k in ['kernel', 'p0', 'p1', 'p2']:
        ap.add_argument(f'--{k}-sha256', dest=f'{k}_sha256')
    ap.add_argument('--workdir'); ap.add_argument('--keep', action='store_true')
    args = ap.parse_args()
    paths = {k: pathlib.Path(getattr(args, k)) for k in ['kernel', 'p0', 'p1', 'p2']}
    for k, p in paths.items():
        if not p.is_file(): raise SystemExit(f'missing {k}: {p}')
        want = getattr(args, f'{k}_sha256') or SHA[k]
        got = hashlib.sha256(p.read_bytes()).hexdigest()
        if got != want: raise SystemExit(f'{k} sha mismatch: expected {want} got {got}')
        print(f'{k} sha OK {got[:16]}', flush=True)
    work = pathlib.Path(args.workdir) if args.workdir else pathlib.Path(tempfile.mkdtemp(prefix='theorem-p2-kit-'))
    work.mkdir(parents=True, exist_ok=True)
    results = []
    try: run(work, *[paths[k] for k in ['kernel','p0','p1','p2']], results)
    finally:
        if not args.keep and not args.workdir: shutil.rmtree(work, ignore_errors=True)
    bad = [(n, d) for n, ok, d in results if not ok]
    print('ALL_OK=' + str(not bad))
    for n, d in bad: print('BAD', n, d)
    raise SystemExit(0 if not bad else 1)

if __name__ == '__main__': main()
