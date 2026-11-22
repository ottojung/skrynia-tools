# GATING BLOCKER — pinned P1 WASM lacks push opcodes 16/17/20-23

Status: **BLOCKED for real-binary P1 push E2E. Narrow loopback test substituted.**
Host: marceline-dev. Branch: `antonina/webpush-loopback-integration-marceline-dev-20261122`.
Base: `470000f2ccedd83eb9dad00f256b59f07a6ba127`. PR76/PR322/PR323 source branches untouched.

## What was inspected

Required first gate (from task): *"inspect whether the pinned P1 WASM actually
contains push opcodes 16/17/20-23; fail with explicit blocker if not."*

Opcode map read from this branch's native-service source
(`native-service/src/protocol/wire/mod.rs`):

| opcode | constant |
| --- | --- |
| 16 | OP_PUSH_SUBSCRIBE |
| 17 | OP_PUSH_UNSUBSCRIBE |
| 18 | OP_PUSH_GRANT |
| 19 | OP_PUSH_SENDER_GRANT |
| 20 | OP_HINT_LIST |
| 21 | OP_HINT_READ |
| 22 | OP_HINT_ACK |
| 23 | OP_PUSH_SUB_FETCH |

Push-enabled **source** exists here and in `skrynia-tools` commit
`a4717fc8fb0a39f7cab3b0bb09ac5751545b06f3`. But **no compiled pinned P1 WASM
available in the workspace contains the push-flow code**. Reproducible audit:

```
cd loopback-integration && python3 audit/pinned_p1_push_opcode_audit.py ; echo $?
# exit 2 (BLOCKER)
```

Exact SHA256 of every candidate pinned P1 artifact (all `NO_PUSH_SIGNATURES`):

| sha256 | bytes | artifact |
| --- | --- | --- |
| `ba6753519aafefcfdb1e11ef892911e763a114062ee8a4e494d38a29a2a30fec` | 133198 | `e2e-aND9NUduGFHjshzPKhHuox-20261122/p1_fresh_ba675351.wasm` (built from `dd234ee6637fc9fbd8568d1ffa4b16150dcb12f9`, pre-push) |
| `6fb58853c908a9265c7ec0425cba2d75e74d0f34b3c9ee822d5be104f47b3699` | 133213 | kit sizedread P1 (receipt-pinned) |
| `83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a` | 133168 | kit p1clean P1 |
| `d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2` | 133129 | kit aa6333a P1 |
| `882fe38434d136b9434ace26e987c2caa324583cfa9a3200262cbaddb17fd562` | 133101 | kit current P1 |
| `7ce4cac5ac69a21084e3f0eb3b9861d6914f511f7bd075d64a1931553e18c4e5` | 124409 | `skrynia-tools-v3-crossbuild` native service |
| `32225284344bd209f55d440b264659be79b836e59b9908956eb82b9094871e0d` | 65807 | `skrynia-tools/target` native service |

Method: scan every candidate WASM for push-flow signature strings
(`push`, `hint`, `sub_fetch`, `subscribe`, `unsubscribe`, `webpush`). A Rust
native service compiled from push-enabled source retains these in panic/format
strings. None are present. A naive `i32.const` byte count is deliberately **not**
used as proof (it is ambiguous); the source commit check corroborates it:
`dd234ee` (source that built `ba675351`) has no push opcodes in native-service.

Supporting pinned binaries available and reused unchanged:
kernel `3508938239c4b7f27591000e2d1d1d36f3b3cc1f976c470fe0e5f84274e3176d`
(receipt) / `8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f`
(prior E2E), dumb P0 `d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa`.

## Consequence

A real-binary E2E that connects the generic kernel + externally supplied
Program0 + **the pinned P1 WASM** + native trusted edge to a receiver **cannot
execute subscribe/unsubscribe/hint/ACK** with these artifacts. Claiming it would
be false. This branch therefore does **not** claim full-stack acceptance.

## Unblock condition (explicit)

Build a fresh `skrynia_native_service.wasm` from push-enabled source
(`a4717fc`/this branch) with the pinned sysroot/ABI, record its SHA256, confirm
the audit above reports `PUSH_PRESENT`, then extend the harness to drive the
real kernel + dumb P0 + that P1 + trusted edge over real HTTP. No such build is
performed here (would be a heavy duplicate build and would touch kernel/P0/P1
business logic, which this task forbids).
