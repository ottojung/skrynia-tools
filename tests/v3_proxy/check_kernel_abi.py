#!/usr/bin/env python3
"""Disposable check that the pinned kernel advertises ABI 3.1.

Verifies the /version identity in source and the fixed import namespace in
the compiled binary. No deployment state, credentials or production data.
"""
from __future__ import annotations
import argparse
import hashlib
import pathlib

ABI = "skrynia_v3_1"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--kernel-source", type=pathlib.Path, required=True)
    ap.add_argument("--kernel-bin", type=pathlib.Path, required=True)
    args = ap.parse_args()
    version = (args.kernel_source / "rust-server/src/sys/version.rs").read_text()
    if f'"abi": "{ABI}"' not in version:
        raise SystemExit("kernel /version does not advertise skrynia_v3_1")
    binary = args.kernel_bin.read_bytes()
    if ABI.encode() not in binary:
        raise SystemExit("compiled kernel has no skrynia_v3_1 import namespace")
    print("KERNEL_ABI_V3_1_OK", ABI, hashlib.sha256(binary).hexdigest())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
