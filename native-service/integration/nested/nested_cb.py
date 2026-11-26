#!/usr/bin/env python3
"""Real compiled P0/P1/P2 callback and error checks; disposable roots only."""
import argparse
import hashlib
import pathlib
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / 'examples/theorem_lab_feedback_p2/tests'))
import real_kit_p2 as h


def digest(data):
    return hashlib.sha256(data).digest()


def run(args, work):
    token, operator, admin, bad_admin = (
        digest(x) for x in (b'probe-token', b'probe-operator', b'probe-admin', b'bad-admin')
    )
    iid, bad_iid = digest(admin), digest(bad_admin)
    srv = h.start(work / 'state', h.free_port(), args.kernel, args.p0, args.p1, token, operator)
    results = []
    try:
        def register(path, capability):
            return srv.send(b'\x01\x0f' + h.put_id(operator) + h.put_id(capability) +
                            h.put_blob(path.read_bytes()))

        def invoke(data, ident=iid):
            return srv.send(b'\x01\x03' + h.put_id(ident) + h.put_blob(data))

        def record(name, actual, expected):
            h.rec(results, name, actual == expected, actual)

        d = register(args.probe, admin)
        record('registered-probe', d.get('kind'), 'ok')
        record('ordinary-echo', invoke(b'\x00hello'), {'kind': 'ok', 'body': b'hello'})
        key, value = b'callback-probe', b'P1-OWNED-CB-ROUNDTRIP-OK'
        get = b'\x01\x07' + h.put_id(iid) + h.put_blob(key)
        put = (b'\x01\x06' + h.put_id(iid) + h.put_blob(key) + b'\x00' + h.put_blob(value))
        d = invoke(b'\x01' + put)
        record('callback-put', h.dec(d.get('body', b'')), {'kind': 'ok', 'body': digest(value)})
        d = invoke(b'\x01' + get)
        record('callback-get', h.dec(d.get('body', b'')), {'kind': 'ok', 'body': value})
        record('persisted-value', srv.send(get), {'kind': 'ok', 'body': value})
        record('incomplete-frame-fails', invoke(b'\x02'), {'kind': 'err', 'code': 4})
        record('sticky-invalid-callback-fails', invoke(b'\x03'), {'kind': 'err', 'code': 4})
        record('unknown-op', invoke(b'\x04'), {'kind': 'ok', 'body': b'\xfd'})
        record('register-invalid-probe', register(args.invalid, bad_admin).get('kind'), 'ok')
        record('malformed-import-fails', invoke(b'', bad_iid), {'kind': 'err', 'code': 4})
        record('subsequent-request-recovers', invoke(b'\x00ok'), {'kind': 'ok', 'body': b'ok'})
        d = srv.send(b'\x01\x03' + h.put_id(iid) + h.put_blob(b'\x00bad'),
                     token=digest(b'wrong-bearer'))
        h.rec(results, 'wrong-bearer-denied', d.get('kind') != 'ok', d)
    finally:
        srv.stop()
    print('NESTED_CB_' + ('PASS' if all(ok for _, ok, _ in results) else 'FAIL'))
    return all(ok for _, ok, _ in results)


def main():
    ap = argparse.ArgumentParser()
    for name in ['kernel', 'p0', 'p1', 'probe', 'invalid']:
        ap.add_argument('--' + name, required=True, type=pathlib.Path)
        ap.add_argument('--' + name + '-sha256', required=True)
    args = ap.parse_args()
    for name in ['kernel', 'p0', 'p1', 'probe', 'invalid']:
        path = getattr(args, name).resolve()
        got = hashlib.sha256(path.read_bytes()).hexdigest()
        if got != getattr(args, name + '_sha256'):
            raise SystemExit(f'{name}: hash mismatch: {got}')
        setattr(args, name, path)
    with tempfile.TemporaryDirectory(prefix='nested-callback-') as work:
        return 0 if run(args, pathlib.Path(work)) else 1


if __name__ == '__main__':
    sys.exit(main())
