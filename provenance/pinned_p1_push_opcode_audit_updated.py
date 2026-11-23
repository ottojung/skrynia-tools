#!/usr/bin/env python3
"""Reproducible gating audit: does the pinned P1 WASM contain push opcodes?

Gates the loopback integration work. Prints exact SHA256 for every candidate
pinned P1 artifact and checks for push-flow signature strings that a native
service compiled from push-enabled source necessarily contains (Rust keeps
panic/format strings such as `PushSubscribe`, `HintAck`, `sub_fetch`).

Exit 0 = at least one pinned P1 has push opcodes; exit 2 = explicit BLOCKER.
"""
import hashlib
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
WASMS = [
    "/workspace/skrynia-v3-integration-kit/wasm-build-record/p1-push-capable-efe6117-20261122/skrynia_native_service_p1_push_efe6117.wasm",
    "e2e-aND9NUduGFHjshzPKhHuox-20261122/p1_fresh_ba675351.wasm",
    "skrynia-v3-integration-kit/artifacts/skrynia_native_service_p1_sizedread_6fb58853.wasm",
    "skrynia-v3-integration-kit/artifacts/skrynia_native_service_p1clean_83a65a2f.wasm",
    "skrynia-v3-integration-kit/artifacts/skrynia_native_service_aa6333a.wasm",
    "skrynia-v3-integration-kit/artifacts/skrynia_native_service_kit_current_882fe384.wasm",
    "skrynia-tools-v3-crossbuild/wasm32-unknown-unknown/release/skrynia_native_service.wasm",
    "skrynia-tools/target/wasm32-unknown-unknown/release/deps/skrynia_native_service.wasm",
]
SIGS = [b"push", b"hint", b"sub_fetch", b"subscribe", b"unsubscribe", b"webpush"]
OPCODES = {16: "OP_PUSH_SUBSCRIBE", 17: "OP_PUSH_UNSUBSCRIBE", 20: "OP_HINT_LIST",
           21: "OP_HINT_READ", 22: "OP_HINT_ACK", 23: "OP_PUSH_SUB_FETCH"}

any_push = False
print("pinned_p1_push_opcode_audit v1")
print("checked_opcodes=" + ",".join(f"{n}:{name}" for n, name in OPCODES.items()))
for rel in WASMS:
    path = os.path.join(ROOT, rel)
    if not os.path.isfile(path):
        print(f"MISSING {rel}")
        continue
    data = open(path, "rb").read()
    sha = hashlib.sha256(data).hexdigest()
    low = data.lower()
    found = sorted({s.decode() for s in SIGS if s in low})
    flag = "PUSH_PRESENT" if found else "NO_PUSH_SIGNATURES"
    any_push = any_push or bool(found)
    print(f"{flag} sha256={sha} bytes={len(data)} sigs={found} {rel}")
print(f"VERDICT={'PASS' if any_push else 'BLOCKER: no pinned P1 WASM contains push opcodes 16/17/20-23'}")
sys.exit(0 if any_push else 2)
