#!/usr/bin/env python3
"""Structural check of the compiled ordinary P1 WASM against the target ABI.

Disposable test harness only: verifies the guest surface (one supplied
function import, nested `_start` entry, no legacy skrynia imports). It does
not claim runtime acceptance or old-ABI qualification.
"""
import sys


def leb(data, at):
    value = 0
    shift = 0
    while True:
        byte = data[at]
        at += 1
        value |= (byte & 0x7F) << shift
        if byte < 0x80:
            return value, at
        shift += 7


def name(data, at):
    size, at = leb(data, at)
    return data[at:at + size].decode(), at + size


def parse(path):
    data = open(path, "rb").read()
    assert data[:4] == b"\0asm", "not wasm"
    at = 8
    imports, exports = [], []
    while at < len(data):
        section = data[at]
        at += 1
        size, at = leb(data, at)
        body = data[at:at + size]
        at += size
        if section == 2:
            count, p = leb(body, 0)
            for _ in range(count):
                module, p = name(body, p)
                field, p = name(body, p)
                kind = body[p]
                p += 1
                _, p = leb(body, p) if kind in (0, 1, 2) else (0, p + 1)
                imports.append((module, field, kind))
        if section == 7:
            count, p = leb(body, 0)
            for _ in range(count):
                field, p = name(body, p)
                kind = body[p]
                p += 1
                _, p = leb(body, p)
                exports.append((field, kind))
    return imports, exports


def main():
    imports, exports = parse(sys.argv[1])
    banned = {"read", "write", "read1", "write1", "eval", "eval_buffered",
              "eval_delegate", "fs_delegate_scope", "invoke"}
    ok = imports == [("p0", "invoke", 0)]
    ok &= any(field == "_start" and kind == 0 for field, kind in exports)
    ok &= not any(module == "skrynia" for module, _, _ in imports)
    ok &= not any(field in banned and module != "p0" for module, field, _ in imports)
    print("imports:", imports)
    print("exports:", [field for field, _ in exports])
    print("TARGET_GUEST_SURFACE_PASS" if ok else "TARGET_GUEST_SURFACE_FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
