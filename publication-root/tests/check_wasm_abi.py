#!/usr/bin/env python3
"""Verify real compiled WASM linkage under the ABI 3.1 fixed namespace.

The dedicated root imports the publish/unpublish named FFI and only fixed
imports from `skrynia_v3_1`; the dumb P0 imports no named FFI. Parses the WASM
import/export/type sections directly; no external tools needed.
Usage: check_wasm_abi.py --root <publication-root.wasm> --generic-p0 <program0.wasm>
"""
from __future__ import annotations
import argparse
import pathlib
import sys

FIXED_MODULE = "skrynia_v3_1"
FFI_MODULE = "syscall"


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


def inspect(path: str):
    types = []
    imports = {}
    exports = {}
    for sid, body in sections(pathlib.Path(path).read_bytes()):
        if sid == 1:
            count, at = varuint(body, 0)
            for _ in range(count):
                assert body[at] == 0x60
                params, at = varuint(body, at + 1)
                for _ in range(params):
                    _, at = varuint(body, at)
                results, at = varuint(body, at)
                for _ in range(results):
                    _, at = varuint(body, at)
                types.append((params, results))
        elif sid == 2:
            count, at = varuint(body, 0)
            for _ in range(count):
                module, at = name(body, at)
                field, at = name(body, at)
                kind = body[at]
                at += 1
                if kind == 0:
                    index, at = varuint(body, at)
                    imports[(module, field)] = types[index]
                elif kind == 1:
                    for _ in range(4):
                        _, at = varuint(body, at)
                elif kind == 2:
                    for _ in range(3):
                        _, at = varuint(body, at)
                elif kind == 3:
                    _, at = varuint(body, at)
                    _, at = varuint(body, at)
        elif sid == 7:
            count, at = varuint(body, 0)
            for _ in range(count):
                field, at = name(body, at)
                _, at = varuint(body, at)
                index, at = varuint(body, at)
                exports[field] = index
    return imports, exports


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--generic-p0", required=True)
    args = ap.parse_args()
    imports, exports = inspect(args.root)
    wanted = {(FFI_MODULE, "publish"), (FFI_MODULE, "unpublish")}
    if not wanted.issubset(imports):
        print("root missing syscalls", sorted(wanted - set(imports)), file=sys.stderr)
        return 1
    if any(imports[key] != (1, 1) for key in wanted):
        print("wrong syscall signature", {k: imports[k] for k in wanted}, file=sys.stderr)
        return 1
    if "skrynia_main" not in exports:
        print("root missing skrynia_main export", file=sys.stderr)
        return 1
    if any(module not in (FIXED_MODULE, FFI_MODULE) for module, _ in imports):
        print("root has unexpected import namespace", sorted(imports), file=sys.stderr)
        return 1
    p0_imports, _ = inspect(args.generic_p0)
    if any(module == FFI_MODULE for module, _ in p0_imports):
        print("generic P0 must not import syscalls", file=sys.stderr)
        return 1
    if any(module not in ("skrynia_v3_1", "p0") for module, _ in p0_imports):
        print("generic P0 has unexpected import namespace", sorted(p0_imports), file=sys.stderr)
        return 1
    print("PUBLICATION_WASM_ABI_OK", sorted(imports), "exports", sorted(exports))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
