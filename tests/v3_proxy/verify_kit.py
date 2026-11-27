#!/usr/bin/env python3
"""Verify the exact source and six artifact pins before any CI consumer runs."""
import argparse
import hashlib
from pathlib import Path
import re

FILES = {
    "kernel": "kernel", "p0": "root.wasm", "p1": "service.wasm",
    "edge": "edge", "echo": "echo.wasm", "p2": "theorem_lab_feedback_p2.wasm",
}


def verify(kit, tools_head, kernel_head):
    pins = {}
    for line in (kit / "COMMIT_MANIFEST.txt").read_text().splitlines():
        key, value = line.split("=", 1)
        if key in pins:
            raise ValueError(f"duplicate manifest key {key}")
        pins[key] = value
    expected = {"tools_head": tools_head, "kernel": kernel_head,
                "kernel_head": kernel_head}
    expected.update({f"sha256_{key}": hashlib.sha256((kit / name).read_bytes()).hexdigest()
                     for key, name in FILES.items()})
    if pins != expected:
        raise ValueError("source or artifact manifest mismatch")
    for filename, keys in [("SHA256SUMS", list(FILES)[:-1]), ("SHA256SUMS.p2", ["p2"])]:
        roster = {}
        for line in (kit / filename).read_text().splitlines():
            digest, name = line.split("  ", 1)
            if name in roster or not re.fullmatch(r"[0-9a-f]{64}", digest):
                raise ValueError(f"invalid {filename} entry")
            roster[name] = digest
        want = {FILES[key]: expected[f"sha256_{key}"] for key in keys}
        if roster != want:
            raise ValueError(f"{filename} roster mismatch")
    print(f"Verified six exact artifacts: tools {tools_head}, kernel {kernel_head}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--kit", required=True, type=Path)
    ap.add_argument("--tools-head", required=True)
    ap.add_argument("--kernel-head", required=True)
    args = ap.parse_args()
    verify(args.kit, args.tools_head, args.kernel_head)


if __name__ == "__main__":
    main()
