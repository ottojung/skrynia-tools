#!/usr/bin/env python3
"""Disposable real-kernel check of framed descriptor ops 5-10 through P0.

Boots a compiled K4 kernel with a freshly built target-ABI Program 0 and a
single-import guest that issues MKDIR/OPEN_DISK/WRITE/SYNC/RENAME/DELETE/
OPEN_DIRECTORY/NEXT_ENTRY frames. Fake bearer, isolated temp dirs, no
production. The seeded listing file lives in the operator read root; writes
land in the user root, matching kernel root selection.
"""
from __future__ import annotations

import argparse, hashlib, os, pathlib, shutil, subprocess, sys, tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from dumb_p0_http import Srv, check_sha, free_port, wait_port

TOKEN = hashlib.sha256(b't2-ops5to10-fake-bearer').digest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--kernel', required=True)
    ap.add_argument('--p0', required=True)
    ap.add_argument('--guest', required=True)
    ap.add_argument('--kernel-sha256')
    ap.add_argument('--p0-sha256')
    ap.add_argument('--workdir')
    ap.add_argument('--want', default='OPS5TO10_OK', help='expected guest body')
    ap.add_argument('--label', default='OPS5TO10', help='result label')
    args = ap.parse_args()
    kernel, p0, guest = (pathlib.Path(x) for x in (args.kernel, args.p0, args.guest))
    for path, name in ((kernel, 'kernel'), (p0, 'p0'), (guest, 'guest')):
        if not path.is_file():
            raise SystemExit(f'missing {name}: {path}')
    check_sha(kernel, args.kernel_sha256, 'kernel')
    check_sha(p0, args.p0_sha256, 'p0')
    work = pathlib.Path(args.workdir) if args.workdir else pathlib.Path(
        tempfile.mkdtemp(prefix='t2-ops5to10-'))
    work.mkdir(parents=True, exist_ok=True)
    try:
        port = free_port()
        op, us = work / 'operator', work / 'user'
        (op / 'security').mkdir(parents=True, exist_ok=True)
        (op / 'programs' / 'appdata' / 'p0ops').mkdir(parents=True, exist_ok=True)
        for name in ('active', 'blob', 'ready', 'appdata'):
            (us / 'programs' / name).mkdir(parents=True, exist_ok=True)
        (op / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(TOKEN).digest())
        (op / 'programs' / 'root.wasm').write_bytes(guest.read_bytes())
        (op / 'programs' / 'appdata' / 'p0ops' / 'seed.bin').write_bytes(b'seed')
        env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(p0),
               'SKRYNIA_OPERATOR_DIR': str(op), 'SKRYNIA_USER_DIR': str(us),
               'SKRYNIA_BIND': f'127.0.0.1:{port}'}
        log_path = work / 'server.log'
        log = open(log_path, 'wb')
        proc = subprocess.Popen([str(kernel)], env=env, stdout=log, stderr=subprocess.STDOUT)
        wait_port(port, proc, log_path)
        srv = Srv(work, port, TOKEN, proc, log)
        try:
            _decoded, status, raw = srv.send(b'p0-descriptor-ops', timeout=60)
            ok = raw == args.want.encode()
            print(args.label + '_' + ('PASS' if ok else 'FAIL'), status, raw, flush=True)
            print('RAW', raw.hex(), flush=True)
            return 0 if ok else 1
        finally:
            srv.stop()
    finally:
        if not args.workdir:
            shutil.rmtree(work, ignore_errors=True)


if __name__ == '__main__':
    raise SystemExit(main())
