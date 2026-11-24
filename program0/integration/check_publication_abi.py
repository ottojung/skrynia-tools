#!/usr/bin/env python3
"""Check that a compiled root WASM exports the publication syscall ABI.

Reads the WASM import/type sections directly; no external tool needed.
Usage: check_publication_abi.py --wasm /path/to/root.wasm
"""
from __future__ import annotations
import argparse
import pathlib
import struct
import sys


def varuint(data: bytes, at: int) -> tuple[int, int]:
    value = 0
    shift = 0
    while True:
        byte = data[at]
        at += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, at
        shift += 7
        if shift > 35:
            raise ValueError("varuint too long")


def name(data: bytes, at: int) -> tuple[str, int]:
    length, at = varuint(data, at)
    return data[at:at + length].decode("utf-8"), at + length


def sections(data: bytes):
    if data[:4] != b"\0asm" or data[4:8] != b"\x01\0\0\0":
        raise ValueError("not a wasm32 module")
    at = 8
    while at < len(data):
        sid = data[at]
        size, at = varuint(data, at + 1)
        yield sid, data[at:at + size]
        at += size


def imports(data: bytes) -> dict[tuple[str, str], tuple[int, int]]:
    types: list[tuple[int, int]] = []
    table: dict[tuple[str, str], tuple[int, int]] = {}
    for sid, body in sections(data):
        if sid == 1:
            count, at = varuint(body, 0)
            for _ in range(count):
                if body[at] != 0x60:
                    raise ValueError("unsupported func type")
                params, at = varuint(body, at + 1)
                for _ in range(params):
                    _, at = varuint(body, at)
                results, at = varuint(body, at)
                for _ in range(results):
                    _, at = varuint(body, at)
                types.append((params, results))
        if sid != 2:
            continue
        count, at = varuint(body, 0)
        for _ in range(count):
            module, at = name(body, at)
            field, at = name(body, at)
            kind = body[at]
            at += 1
            if kind == 0:
                index, at = varuint(body, at)
                table[(module, field)] = types[index]
            elif kind == 1:
                _, at = varuint(body, at)
                _, at = varuint(body, at)
                _, at = varuint(body, at)
                _, at = varuint(body, at)
            elif kind == 2:
                _, at = varuint(body, at)
                _, at = varuint(body, at)
                _, at = varuint(body, at)
            elif kind == 3:
                _, at = varuint(body, at)
                _, at = varuint(body, at)
            else:
                raise ValueError("unknown import kind")
    return table


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--wasm", required=True)
    args = ap.parse_args()
    wasm = pathlib.Path(args.wasm)
    table = imports(wasm.read_bytes())
    wanted = {("syscall", "publish"), ("syscall", "unpublish")}
    missing = sorted(wanted - set(table))
    if missing:
        print("MISSING imports:", missing, file=sys.stderr)
        return 1
    for key in sorted(wanted):
        sig = table[key]
        if sig != (1, 1):
            print(f"BAD signature {key}: {sig}", file=sys.stderr)
            return 1
    children = [k for k in table if k[0] != "skrynia" and k[0] != "syscall"]
    if children:
        print("UNEXPECTED import namespace:", children, file=sys.stderr)
        return 1
    print("PUBLICATION_ABI_OK", wasm, sorted(table))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
