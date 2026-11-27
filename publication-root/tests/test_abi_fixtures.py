#!/usr/bin/env python3
"""Self-tests for the compiled-root ABI assertion fixtures.

Generates a minimal WASM binary in memory so the namespace and signature
checks are exercised without a compiled publication root. No kernel, provider
or nginx is involved.
"""
from __future__ import annotations

import pathlib
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
I32, I64 = 0x7F, 0x7E
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
    "sync": ((I32,), (I32,)),
    "mkdir": ((I64, I64, I32), (I32,)),
    "exists": ((I64, I64), (I32,)),
    "is_dir": ((I64, I64), (I32,)),
    "delete": ((I64, I64), (I32,)),
    "rename": ((I64, I64, I64, I64), (I32,)),
    "open_directory": ((I64, I64), (I32,)),
    "next_entry": ((I32, I64, I64), (I32,)),
    "cas1": ((I32, I64, I32, I32), (I32,)),
    "now": ((), (I64,)),
    "secure_random_bits": ((I64, I64), (I32,)),
    "eval": ((I64, I64, I32, I32, I32), (I64,)),
}
REQUIRED = ("open_ram", "open_disk", "read1", "write1", "seek", "mkdir",
            "delete", "sync", "close", "secure_random_bits", "read", "write")


def uv(value: int) -> bytes:
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        out.append(byte | (0x80 if value else 0))
        if not value:
            return bytes(out)


def text(value: str) -> bytes:
    raw = value.encode()
    return uv(len(raw)) + raw


def section(sid: int, body: bytes) -> bytes:
    return bytes((sid,)) + uv(len(body)) + body


def wasm(fixed_module: str = "skrynia_v3_1", read_params=None, exports=True) -> bytes:
    read_params = read_params or TARGET["read"][0]
    inventory = []
    for name in REQUIRED:
        params, results = TARGET[name]
        inventory.append((name, (read_params, results) if name == "read" else (params, results)))
    types = inventory + [("publish", ((I32,), (I32,))), ("unpublish", ((I32,), (I32,)))]
    type_body = uv(len(types))
    for _, (params, results) in types:
        type_body += b"\x60" + uv(len(params)) + bytes(params) + uv(len(results)) + bytes(results)
    imports = [(fixed_module, name, index) for index, (name, _) in enumerate(inventory)]
    imports += [("syscall", "publish", len(types) - 2), ("syscall", "unpublish", len(types) - 1)]
    import_body = uv(len(imports))
    for module, field, index in imports:
        import_body += text(module) + text(field) + b"\x00" + uv(index)
    out = b"\x00asm\x01\x00\x00\x00"
    out += section(1, type_body) + section(2, import_body)
    out += section(5, b"\x01\x00\x01")
    if exports:
        export_body = uv(2) + text("skrynia_main") + b"\x00" + uv(0)
        export_body += text("memory") + b"\x02" + uv(0)
        out += section(7, export_body)
    return out


def run(script: str, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run([sys.executable, str(HERE / script), *args],
                          capture_output=True, text=True)


class AbiFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name) / "root.wasm"
        self.p0 = pathlib.Path(self.tmp.name) / "p0.wasm"
        self.p0.write_bytes(b"\x00asm\x01\x00\x00\x00")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def check(self, data: bytes) -> tuple[subprocess.CompletedProcess, subprocess.CompletedProcess]:
        self.root.write_bytes(data)
        return (run("check_wasm_abi.py", "--root", str(self.root), "--generic-p0", str(self.p0)),
                run("check_target_abi.py", "--root", str(self.root)))

    def test_abi_31_fixed_namespace_and_bulk_signatures_are_accepted(self):
        linkage, target = self.check(wasm())
        self.assertEqual(linkage.returncode, 0, linkage.stderr)
        self.assertEqual(target.returncode, 0, target.stderr)

    def test_retired_namespace_is_rejected(self):
        linkage, target = self.check(wasm(fixed_module="skrynia"))
        self.assertNotEqual(linkage.returncode, 0)
        self.assertNotEqual(target.returncode, 0)

    def test_wrong_bulk_signature_is_rejected(self):
        _, target = self.check(wasm(read_params=(I32, I32, I32)))
        self.assertNotEqual(target.returncode, 0)
        self.assertIn("wrong ABI 3.1 signature", target.stderr)

    def test_missing_memory_export_is_rejected(self):
        _, target = self.check(wasm(exports=False))
        self.assertNotEqual(target.returncode, 0)


if __name__ == "__main__":
    unittest.main()
