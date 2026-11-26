#!/usr/bin/env python3
"""Disposable real-kernel nested P2 callback integration (source-test only).

Boots the compiled K4 kernel with T2's ops-5-to-10 P0 and the size-optimized
T3/T6 P1, then exercises ordinary P1-owned appdata callbacks through a nested
P2 guest that uses only CB_GET/CB_PUT. The interpreter calls P1's own callback
on P1's stack: no P0 relay and no suspended P1, so a callback-dependent P2
cannot deadlock. Registry/appdata fixtures are staged directly on the
disposable host dirs. Artifact paths can be rebound with SKRYNIA_*_WASM and
SKRYNIA_NESTED_WORKDIR; every SHA is re-pinned below. Fake bearer, temp dirs,
no production. Not Theorem Lab end-to-end acceptance and not a packaged P1
module claim.
"""
import hashlib
import os
import pathlib
import struct
import sys

import dumb_p0_http as h

E = pathlib.Path(__file__).resolve().parent
KERNEL = pathlib.Path(os.environ.get('SKRYNIA_KERNEL_WASM',
                                     '/tmp/opencode/t2-evidence/kernel-k4-b8eff72'))
P0 = pathlib.Path(os.environ.get('SKRYNIA_P0_WASM',
                                 '/tmp/opencode/FDGNmUxm-t2-ops5to10/p0-new.wasm'))
P1 = pathlib.Path(os.environ.get('SKRYNIA_P1_WASM', str(E / 'p1-t6-sizeopt.wasm')))
TOKEN = hashlib.sha256(b't3-t6-nested-fake-bearer').digest()
VALUE = b'P1-OWNED-CB-ROUNDTRIP-OK'
HASHES = {
    'kernel': (KERNEL, 'c82ab50285c1b61f45dc24ca2dcf4b33ee13a820601e6a7ec3eea92cd30da43b'),
    'p0': (P0, 'bb3b8afb6bc6999015a9dc667b00733504ae8f703d6634436819a415c0f46c1b'),
    'p1': (P1, 'e9a65426caa5902716c6a07ded6071829980ce4ee4ceab0d42ae6362fca4fcfc'),
}
GUESTS = {
    'echo': 'd8d50017cfc95d3d3059df9603bbfcc23f530251395f9de3eff4832a0fb9b106',
    'cb_roundtrip': 'db73bd25ebfef3390165a8b0dee16c61b5413611adc20f8eb4cf488a804ac9c2',
    'cb_failure': 'f58c3b4a6d51717e7cd84b124f3c6b4b4627362a27d25b587cf8d374fbabb967',
    'malformed_import': 'f789f771e26b0dda79656e0f145ece992e9efb1eef86b0c5a82fc4a3372242bc',
    'unknown_op': '5b8230d75990935408f78f10f2d1ab550d6f72d3f6b2f5aae3cd4fce628bffbb',
}
BLOB_VER = 1
ACTIVE_VER = 3


def hexs(b):
    return b.hex()


def stage(base, name, code, value):
    """Stage an active program and its appdata on the disposable user dir."""
    admin = hashlib.sha256(f't3-t6-admin-{name}'.encode()).digest()
    iid = hashlib.sha256(admin).digest()
    # The kernel resolves all read-only open_disk paths to the operator root.
    programs = base / 'operator' / 'programs'
    (programs / 'blob').mkdir(parents=True, exist_ok=True)
    (programs / 'active').mkdir(parents=True, exist_ok=True)
    ns = programs / 'appdata' / hexs(iid)
    ns.mkdir(parents=True, exist_ok=True)
    code_hash = hashlib.sha256(code).digest()
    (programs / 'blob' / f'{hexs(code_hash)}.bin').write_bytes(bytes([BLOB_VER]) + code)
    active = bytes([ACTIVE_VER, 1]) + code_hash + struct.pack('<Q', 1)
    (programs / 'active' / f'{hexs(iid)}.bin').write_bytes(active)
    key_hash = hashlib.sha256(b'cb').digest()
    (ns / f'{hexs(key_hash)}.bin').write_bytes(value)
    return iid


def main():
    for name, (path, want) in HASHES.items():
        h.check_sha(path, want, name)
    codes = {}
    for name, want in GUESTS.items():
        path = E / f'{name}.wasm'
        h.check_sha(path, want, name)
        codes[name] = path.read_bytes()

    base = E / os.environ.get('SKRYNIA_NESTED_WORKDIR', 'nested-cbproof-work')
    port = h.free_port()
    srv = h.setup(base, port, KERNEL, P0, P1, TOKEN)
    results = []
    try:
        iids = {name: stage(srv.base, name, code, VALUE) for name, code in codes.items()}

        d, _, _ = srv.send(b'\x01\x03' + h.put_id(iids['echo']) +
                           h.put_blob(b't6-echo-ok'), 'invoke-echo', timeout=60)
        h.rec(results, 'nested-echo-via-t6-interpreter',
              d.get('kind') == 'ok' and d.get('body') == b't6-echo-ok', d)

        d, _, _ = srv.send(b'\x01\x03' + h.put_id(iids['cb_roundtrip']) +
                           h.put_blob(b'\x00'), 'invoke-cb-roundtrip', timeout=60)
        inner = h.dec(d.get('body', b'')) if d.get('kind') == 'ok' else d
        h.rec(results, 'p1-owned-appdata-cb-get-put-roundtrip',
              inner.get('kind') == 'ok' and inner.get('body') == VALUE,
              {'outer': d, 'inner': inner})

        d, _, _ = srv.send(b'\x01\x03' + h.put_id(iids['cb_failure']) +
                           h.put_blob(b''), 'invoke-cb-failure', timeout=60)
        h.rec(results, 'sticky-callback-failure-fails-closed',
              d.get('kind') == 'err' and d.get('code') == 4, d)

        d, _, _ = srv.send(b'\x01\x03' + h.put_id(iids['malformed_import']) +
                           h.put_blob(b''), 'invoke-malformed', timeout=60)
        h.rec(results, 'malformed-import-fails-closed',
              d.get('kind') == 'err' and d.get('code') == 4, d)

        d, _, _ = srv.send(b'\x01\x03' + h.put_id(iids['unknown_op']) +
                           h.put_blob(b''), 'invoke-unknown-op', timeout=60)
        # The guest emits the raw low byte of EINVAL (-3); no nested wire frame.
        h.rec(results, 'unknown-op-fails-closed',
              d.get('kind') == 'ok' and d.get('body') == b'\xfd', d)

        wrong = hashlib.sha256(b't3-t6-wrong-bearer').digest()
        d, status, _ = srv.send(b'\x01\x03' + h.put_id(iids['cb_roundtrip']) +
                                h.put_blob(b'\x00'), 'wrong-bearer', timeout=60, token=wrong)
        h.rec(results, 'wrong-bearer-denied', d.get('kind') != 'ok', status)
    finally:
        srv.stop()

    bad = [n for n, ok, _ in results if not ok]
    print('T3_T6_NESTED_CB_' + ('PASS' if not bad else 'FAIL'), flush=True)
    for n in bad:
        print('BAD', n, flush=True)
    return 0 if not bad else 1


if __name__ == '__main__':
    sys.exit(main())
