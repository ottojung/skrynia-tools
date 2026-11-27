#!/usr/bin/env python3
"""Real compiled P0 staging boundary regression on disposable roots."""
import argparse
import hashlib
from pathlib import Path
from tempfile import TemporaryDirectory

from dumb_p0_http import free_port, setup

CAPACITY = 2 * 1024 * 1024 - 1152
TOKEN = hashlib.sha256(b'P0 staging boundary disposable test').digest()


def leb(n):
    out = bytearray()
    while True:
        b = n & 127
        n >>= 7
        out.append(b | (128 if n else 0))
        if not n:
            return bytes(out)


def section(kind, body):
    return bytes([kind]) + leb(len(body)) + body


def name(value):
    return leb(len(value)) + value


# One integer-signature callback import, an empty _start, no linear memory.
SMALL = (
    b'\x00asm\x01\x00\x00\x00'
    + section(1, b'\x02\x60\x03\x7f\x7f\x7f\x01\x7e\x60\x00\x00')
    + section(2, b'\x01' + name(b'p0') + name(b'invoke') + b'\x00\x00')
    + section(3, b'\x01\x01')
    + section(7, b'\x01' + name(b'_start') + b'\x00\x01')
    + section(10, b'\x01\x02\x00\x0b')
)


def padded(size):
    # A valid custom section makes the executable exactly the chosen size.
    prefix = name(b'pad')
    for width in range(1, 6):
        n = size - len(SMALL) - 1 - width
        if n >= len(prefix) and len(leb(n)) == width:
            out = SMALL + section(0, prefix + bytes(n - len(prefix)))
            assert len(out) == size
            return out
    raise ValueError(size)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--kernel', type=Path, required=True)
    parser.add_argument('--p0', type=Path, required=True)
    args = parser.parse_args()
    kernel, p0 = args.kernel.resolve(), args.p0.resolve()
    for label, path in [('kernel', kernel), ('p0', p0)]:
        print(label, hashlib.sha256(path.read_bytes()).hexdigest(), flush=True)
    with TemporaryDirectory(prefix='p0-stage-boundary-') as temp:
        root = Path(temp)
        fixture = root / 'small.wasm'
        fixture.write_bytes(SMALL)
        srv = setup(root / 'stack', free_port(), kernel, p0, fixture, TOKEN)
        try:
            guest = srv.base / 'operator/programs/root.wasm'
            for case, code, want in [
                ('small', SMALL, 200),
                ('exact', padded(CAPACITY), 200),
                ('one-over', padded(CAPACITY + 1), 500),
                ('recovery', SMALL, 200),
            ]:
                guest.write_bytes(code)
                _, status, body = srv.send(b'', timeout=60)
                actual = int(status.split()[1])
                assert actual == want, (case, len(code), status, want)
                if actual == 200:
                    assert body == b'', (case, len(body))
                print('PASS', case, len(code), actual, flush=True)
        finally:
            srv.stop()


if __name__ == '__main__':
    main()
