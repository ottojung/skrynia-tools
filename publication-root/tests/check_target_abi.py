#!/usr/bin/env python3
"""Assert the compiled root uses only ABI 3.1 fixed imports.

Checks exact WASM value types against the owner-approved 21-function
`skrynia_v3_1` inventory and the preserved typed (i32)->i32 publish/unpublish
provider signature. The retired `skrynia` namespace, old-pointer and retired
imports are rejected; buffer pointers and lengths must be u64 (WASM i64) and
bulk read/write must return an i64 byte count.
"""
from __future__ import annotations
import argparse
import pathlib
import sys

from check_wasm_abi import FIXED_MODULE, FFI_MODULE, sections, varuint, name

I32, I64 = 0x7F, 0x7E
FIXED_NS = "skrynia_v3_1"

TARGET = {
    "open_ram": ((), (I32,)),
    "open_disk": ((I64, I64, I32), (I32,)),
    "close": ((I32,), (I32,)),
    "read": ((I32, I64, I64), (I64,)),
    "write": ((I32, I64, I64), (I64,)),
    "read1": ((I32,), (I32,)),
    "write1": ((I32, I32), (I32,)),
    "seek": ((I32, I64), (I64,)),
    "truncate": ((I32, I64), (I32,)),
    "mkdir": ((I64, I64, I32), (I32,)),
    "exists": ((I64, I64), (I32,)),
    "is_dir": ((I64, I64), (I32,)),
    "delete": ((I64, I64), (I32,)),
    "sync": ((I32,), (I32,)),
    "rename": ((I64, I64, I64, I64), (I32,)),
    "open_directory": ((I64, I64), (I32,)),
    "next_entry": ((I32, I64, I64), (I32,)),
    "cas1": ((I32, I64, I32, I32), (I32,)),
    "now": ((), (I64,)),
    "secure_random_bits": ((I64, I64), (I32,)),
    "eval": ((I64, I64, I32, I32, I32), (I64,)),
}
assert len(TARGET) == 21
REQUIRED = {"open_disk", "close", "read", "write", "mkdir", "delete", "sync",
            "secure_random_bits"}


def inspect(path: str):
    types, imports, exports = [], {}, {}
    for sid, body in sections(pathlib.Path(path).read_bytes()):
        if sid == 1:
            count, at = varuint(body, 0)
            for _ in range(count):
                assert body[at] == 0x60
                n, at = varuint(body, at + 1)
                params = tuple(body[at:at + n]); at += n
                n, at = varuint(body, at)
                results = tuple(body[at:at + n]); at += n
                types.append((params, results))
        elif sid == 2:
            count, at = varuint(body, 0)
            for _ in range(count):
                module, at = name(body, at)
                field, at = name(body, at)
                kind = body[at]; at += 1
                if kind == 0:
                    index, at = varuint(body, at)
                    imports[(module, field)] = types[index]
                elif kind == 1:
                    for _ in range(4): _, at = varuint(body, at)
                elif kind == 2:
                    for _ in range(3): _, at = varuint(body, at)
                elif kind == 3:
                    _, at = varuint(body, at); _, at = varuint(body, at)
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
    args = ap.parse_args()
    imports, exports = inspect(args.root)
    fixed = {k: v for (m, k), v in imports.items() if m == FIXED_MODULE}
    for (module, field), sig in imports.items():
        if module not in (FIXED_MODULE, FFI_MODULE):
            print("foreign import namespace", module, field, file=sys.stderr)
            return 1
    for import_name, sig in fixed.items():
        if import_name not in TARGET:
            print("forbidden fixed import", import_name, file=sys.stderr)
            return 1
        if sig != TARGET[import_name]:
            print("wrong ABI 3.1 signature", import_name, sig, file=sys.stderr)
            return 1
    syscall = {k: v for (m, k), v in imports.items() if m == FFI_MODULE}
    if set(syscall) != {"publish", "unpublish"}:
        print("unexpected syscall namespace", sorted(syscall), file=sys.stderr)
        return 1
    if any(sig != ((I32,), (I32,)) for sig in syscall.values()):
        print("publish/unpublish must be (i32)->i32", file=sys.stderr)
        return 1
    if not REQUIRED.issubset(fixed):
        print("missing target imports", sorted(REQUIRED - set(fixed)), file=sys.stderr)
        return 1
    if "skrynia_main" not in exports or "memory" not in exports:
        print("root must export skrynia_main and memory", file=sys.stderr)
        return 1
    print("TARGET_WASM_ABI_OK", FIXED_NS, sorted(fixed), "syscall", sorted(syscall))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
