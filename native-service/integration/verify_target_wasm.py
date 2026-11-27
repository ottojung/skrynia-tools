#!/usr/bin/env python3
"""Check the ordinary P1's exact compiled import and entrypoint signatures.

This is a surface check, not a full WASM validator. The kernel also validates
the entire module before running it. No application or engine policy here.
"""
from pathlib import Path
import sys

I32, I64 = 0x7f, 0x7e
VALUE_TYPES = {0x7f, 0x7e, 0x7d, 0x7c, 0x7b, 0x70, 0x6f}


class Reader:
    def __init__(self, data):
        self.data = data
        self.at = 0

    def take(self, n):
        if n > len(self.data) - self.at:
            raise ValueError("truncated WASM field")
        out = self.data[self.at:self.at + n]
        self.at += n
        return out

    def byte(self):
        return self.take(1)[0]

    def u32(self):
        value = 0
        for shift in range(0, 35, 7):
            b = self.byte()
            value |= (b & 0x7f) << shift
            if not b & 0x80:
                if value > 0xffffffff:
                    raise ValueError("WASM u32 overflow")
                return value
        raise ValueError("overlong WASM u32")

    def name(self):
        return self.take(self.u32()).decode("utf-8")

    def vector(self, item):
        return [item() for _ in range(self.u32())]

    def types(self):
        values = tuple(self.take(self.u32()))
        if not set(values) <= VALUE_TYPES:
            raise ValueError("unsupported WASM value type")
        return values

    def finish(self):
        if self.at != len(self.data):
            raise ValueError("trailing bytes in WASM section")


def parse(data):
    reader = Reader(data)
    if reader.take(8) != b"\0asm\x01\0\0\0":
        raise ValueError("not a version-1 WASM module")
    sections = {}
    while reader.at < len(data):
        sid = reader.byte()
        body = reader.take(reader.u32())
        if sid != 0:
            if sid in sections:
                raise ValueError(f"duplicate WASM section {sid}")
            sections[sid] = body
    if 8 in sections:
        raise ValueError("P1 must not define a start section as well as _start")
    r = Reader(sections.get(1, b"\0"))

    def signature():
        if r.byte() != 0x60:
            raise ValueError("unsupported WASM type section")
        return r.types(), r.types()

    types = r.vector(signature)
    r.finish()
    r = Reader(sections.get(2, b"\0"))
    if r.u32() != 1:
        raise ValueError("P1 must have exactly one import")
    module, field, kind = r.name(), r.name(), r.byte()
    if (module, field, kind) != ("p0", "invoke", 0):
        raise ValueError("P1 import must be function p0.invoke")
    imported = r.u32()
    r.finish()
    if types[imported] != ((I32, I32, I32), (I64,)):
        raise ValueError("p0.invoke must have type (i32,i32,i32)->i64")
    r = Reader(sections.get(3, b"\0"))
    functions = [imported] + r.vector(r.u32)
    r.finish()
    r = Reader(sections.get(7, b"\0"))
    exports = r.vector(lambda: (r.name(), r.byte(), r.u32()))
    r.finish()
    starts = [(kind, index) for field, kind, index in exports if field == "_start"]
    if len(starts) != 1 or starts[0][0] != 0:
        raise ValueError("P1 must export one _start function")
    index = starts[0][1]
    if index == 0 or types[functions[index]] != ((), ()):
        raise ValueError("P1 _start must be a defined ()->() function")
    return [field for field, _, _ in exports]


def main():
    try:
        exports = parse(Path(sys.argv[1]).read_bytes())
    except (ValueError, IndexError, OSError) as err:
        print(f"TARGET_GUEST_SURFACE_FAIL: {err}", file=sys.stderr)
        return 1
    print("imports: p0.invoke (i32,i32,i32)->i64")
    print("exports:", exports)
    print("TARGET_GUEST_SURFACE_PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
