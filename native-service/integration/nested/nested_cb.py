#!/usr/bin/env python3
"""Disposable real-kernel nested P2 callback integration (source-test only).

Boots the compiled K4 kernel with T2's target P0 and the size-optimized T3/T6
P1, then exercises ordinary P1-owned appdata callbacks through a nested P2
guest that uses only CB_GET/CB_PUT. Registry/appdata fixtures are staged
directly on the disposable host dirs because T2's final P0 does not relay
MKDIR/DELETE/RENAME/SYNC (ops 5-10); only read-path gateway ops are used at
run time. Fake bearer, temp dirs, no production. Not Theorem Lab end-to-end
acceptance and not a packaged P1 module claim.
"""
import hashlib
import pathlib
import struct
import sys

import dumb_p0_http as h

E = pathlib.Path(__file__).resolve().parent
KERNEL = pathlib.Path('/tmp/opencode/t2-evidence/kernel-k4-b8eff72')
P0 = pathlib.Path('/tmp/opencode/t2-program0-wasm-target/wasm32-unknown-unknown/release/skrynia_program0.wasm')
P1 = E / 'p1-t6-sizeopt.wasm'
TOKEN = hashlib.sha256(b't3-t6-nested-fake-bearer').digest()
VALUE = b'P1-OWNED-CB-ROUNDTRIP-OK'
HASHES = {
    'kernel': (KERNEL, 'c82ab50285c1b61f45dc24ca2dcf4b33ee13a820601e6a7ec3eea92cd30da43b'),
    'p0': (P0, '985d238b72a6554001f9642240f5d1a2d28fc1c23d9075fc3a61e988a7bd33bb'),
    'p1': (P1, 'e9a65426caa5902716c6a07ded6071829980ce4ee4ceab0d42ae6362fca4fcfc'),
}
GUESTS = {
    'echo': 'd8d50017cfc95d3d3059df9603bbfcc23f530251395f9de3eff4832a0fb9b106',
    'cb_roundtrip': 'db73bd25ebfef3390165a8b0dee16c61b5413611adc20f8eb4cf488a804ac9c2',
    'cb_failure': 'f58c3b4a6d51717e7cd84b124f3c6b4b4627362a27d25b587cf8d374fbabb967',
    'malformed_import': 'f789f771e26b0dda79656e0f145ece992e9efb1eef86b0c5a82fc4a3372242bc',
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

    base = E / 'nested-work'
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
