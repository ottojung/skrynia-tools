#!/usr/bin/env python3
"""Deterministic real-kernel check of the T10 P1-wide userland lease.

Boots a disposable K10 kernel with the target-ABI Program 0 and a single-import
role guest. Two requests are forced to overlap with filesystem markers: the
holder acquires the lease before the contender is sent, so only a genuinely
shared lease can make the contender see contention. A third scenario traps
while holding the lease and verifies request-exit recovery. Fake bearer,
bounded temp dirs, no production. The same script run with the pre-fix P0
prints FAIL for both invariants.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from dumb_p0_http import Srv, check_sha, free_port, wait_port

TOKEN = hashlib.sha256(b't10-shared-lease-dummy-bearer').digest()
# Guest paths carry the kernel root token; host paths are under the user root.
BASE = pathlib.Path('programs/appdata/t10lock')
HELD = BASE / 'held.bin'
RESULT = BASE / 'result.bin'
READY = BASE / 'ready.bin'
GO = BASE / 'go.bin'


def boot(root: pathlib.Path, kernel: pathlib.Path, p0: pathlib.Path, guest: pathlib.Path):
    root.mkdir(parents=True, exist_ok=True)
    op, us = root / 'operator', root / 'user'
    (op / 'security').mkdir(parents=True, exist_ok=True)
    (op / 'programs').mkdir(parents=True, exist_ok=True)
    (us / BASE).mkdir(parents=True, exist_ok=True)
    (op / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(TOKEN).digest())
    (op / 'programs' / 'root.wasm').write_bytes(guest.read_bytes())
    port = free_port()
    env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(p0),
           'SKRYNIA_OPERATOR_DIR': str(op), 'SKRYNIA_USER_DIR': str(us),
           'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    log_path = root / 'server.log'
    log = open(log_path, 'wb')
    proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
    wait_port(port, proc, log_path)
    return Srv(root, port, TOKEN, proc, log), us


def post(srv: Srv, role: bytes):
    _d, status, raw = srv.send(role, timeout=90)
    return status.startswith('HTTP/1.1 200'), raw


def wait_file(path: pathlib.Path, timeout: float = 30.0) -> bool:
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if path.is_file():
            return True
        time.sleep(0.05)
    return False


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--kernel', required=True)
    ap.add_argument('--p0', required=True)
    ap.add_argument('--guest', required=True)
    ap.add_argument('--kernel-sha256')
    ap.add_argument('--p0-sha256')
    ap.add_argument('--workdir')
    args = ap.parse_args()
    kernel, p0, guest = (pathlib.Path(x) for x in (args.kernel, args.p0, args.guest))
    for path, name in ((kernel, 'kernel'), (p0, 'p0'), (guest, 'guest')):
        if not path.is_file():
            raise SystemExit(f'missing {name}: {path}')
    check_sha(kernel, args.kernel_sha256, 'kernel')
    check_sha(p0, args.p0_sha256, 'p0')
    work = pathlib.Path(args.workdir) if args.workdir else pathlib.Path(
        tempfile.mkdtemp(prefix='t10-shared-lease-'))
    work.mkdir(parents=True, exist_ok=True)
    results = []
    try:
        srv, us = boot(work / 'contend', kernel, p0, guest)
        try:
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                holder = pool.submit(post, srv, b'H')
                if wait_file(us / HELD):
                    contender = pool.submit(post, srv, b'C')
                    if wait_file(us / READY):
                        # Both parties are in position. GO is the only thing
                        # that lets the contender CAS; the holder still holds
                        # the lease until RESULT appears, so exclusion is
                        # deterministic and not a reflection of CPU speed.
                        (us / GO).write_bytes(b'1')
                        if wait_file(us / RESULT):
                            observed = (us / RESULT).read_bytes()
                        else:
                            observed = b'NO_RESULT'
                    else:
                        observed = b'NO_CONTENDER_READY'
                    c_ok, c_raw = contender.result(timeout=120)
                else:
                    observed = b'NO_HOLDER_MARKER'
                    c_ok, c_raw = False, b'skipped'
                h_ok, h_raw = holder.result(timeout=120)
            results.append(('holder-held-and-released', h_ok and h_raw == b'LOCK_HELD_RELEASED', h_raw[:80]))
            results.append(('contender-excluded', c_ok and observed == b'LOCK_CONTENDED' and c_raw == observed, observed[:80]))
        finally:
            srv.stop()

        srv, us = boot(work / 'trap', kernel, p0, guest)
        try:
            t_ok, t_raw = post(srv, b'T')
            results.append(('trapping-holder-fails', not t_ok, t_raw[:80]))
            if not wait_file(us / HELD):
                raise RuntimeError('trapping holder never entered critical section')
            p_ok, p_raw = post(srv, b'P')
            results.append(('trap-lease-recovered', p_ok and p_raw == b'LOCK_RECOVERED', p_raw[:80]))
        finally:
            srv.stop()
    finally:
        if not args.workdir:
            shutil.rmtree(work, ignore_errors=True)
    bad = [(n, d) for n, ok, d in results if not ok]
    for n, ok, d in results:
        print(('PASS' if ok else 'FAIL'), n, d, flush=True)
    print('ALL_OK=' + str(not bad), flush=True)
    return 0 if not bad else 1


if __name__ == '__main__':
    raise SystemExit(main())
