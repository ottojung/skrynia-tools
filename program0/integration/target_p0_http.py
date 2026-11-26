#!/usr/bin/env python3
"""Disposable real-kernel validation of target-ABI Program 0.

Fake bearer only. Boots an isolated compiled kernel, stages a minimal
single-invoke child, and checks byte echo, token denial, trap recovery and
concurrent per-request RAM isolation over HTTP. No production or real secret.
"""
from __future__ import annotations
import argparse, concurrent.futures, hashlib, os, pathlib, shutil, socket
import subprocess, sys, tempfile, time
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from dumb_p0_http import Srv, check_sha, dechunk, dec, free_port, wait_port

TOKEN = hashlib.sha256(b't2-target-p0-dummy-bearer').digest()

def stage(guest: pathlib.Path, srv: Srv):
    (srv.base / 'operator' / 'programs' / 'root.wasm').write_bytes(guest.read_bytes())

def post(srv: Srv, payload: bytes, token: bytes | None = None):
    _d, http_status, raw = srv.send(payload, token=token, timeout=60)
    return http_status.startswith('HTTP/1.1 200'), raw

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--kernel', required=True)
    ap.add_argument('--p0', required=True)
    ap.add_argument('--guest', required=True)
    ap.add_argument('--trap-guest', required=True)
    ap.add_argument('--kernel-sha256')
    ap.add_argument('--p0-sha256')
    ap.add_argument('--workdir')
    args = ap.parse_args()
    kernel, p0 = pathlib.Path(args.kernel), pathlib.Path(args.p0)
    guest, trap = pathlib.Path(args.guest), pathlib.Path(args.trap_guest)
    for p, n in [(kernel, 'kernel'), (p0, 'p0'), (guest, 'guest'), (trap, 'trap-guest')]:
        if not p.is_file(): raise SystemExit(f'missing {n}: {p}')
    check_sha(kernel, args.kernel_sha256, 'kernel')
    check_sha(p0, args.p0_sha256, 'p0')
    work = pathlib.Path(args.workdir) if args.workdir else pathlib.Path(
        tempfile.mkdtemp(prefix='t2-target-p0-'))
    work.mkdir(parents=True, exist_ok=True)
    results = []
    try:
        port = free_port()
        op, us = work / 'operator', work / 'user'
        (op / 'security').mkdir(parents=True, exist_ok=True)
        (op / 'programs').mkdir(parents=True, exist_ok=True)
        for s in ('active', 'blob', 'ready', 'appdata'):
            (us / 'programs' / s).mkdir(parents=True, exist_ok=True)
        (op / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(TOKEN).digest())
        stage(guest, Srv(work, port, TOKEN, None, None))
        env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(p0),
               'SKRYNIA_OPERATOR_DIR': str(op), 'SKRYNIA_USER_DIR': str(us),
               'SKRYNIA_BIND': f'127.0.0.1:{port}'}
        log_path = work / 'server.log'
        log = open(log_path, 'wb')
        proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
        wait_port(port, proc, log_path)
        srv = Srv(work, port, TOKEN, proc, log)
        try:
            payload = b't2-invoke-echo-' * 8
            ok, body = post(srv, payload)
            results.append(('echo', ok and body == payload, body[:80]))
            results.append(('wrong-token', not post(srv, b'x', b'\x00' * 32)[0], 'denied'))
            stage(trap, srv)
            results.append(('child-trap', not post(srv, b'boom')[0], 'trapped'))
            stage(guest, srv)
            ok, body = post(srv, payload)
            results.append(('recovery-no-replay', ok and body == payload, body[:80]))
            def one(i: int):
                data = f'parallel-{i:03d}-'.encode() * 20
                return data, post(srv, data)
            with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
                for data, got in pool.map(one, range(16)):
                    results.append((f'isolated-{data[:9].decode()}',
                                    got[0] and got[1] == data, got[1][:40]))
        finally:
            srv.stop()
    finally:
        if not args.workdir: shutil.rmtree(work, ignore_errors=True)
    bad = [(n, d) for n, ok, d in results if not ok]
    print('ALL_OK=' + str(not bad))
    for n, d in bad: print('BAD', n, d)
    return 0 if not bad else 1

if __name__ == '__main__':
    raise SystemExit(main())
