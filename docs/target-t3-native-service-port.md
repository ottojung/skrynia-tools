# T3: ordinary P1 native-service target-ABI port

Derivative architecture for owner head Skrynia `0ccc90f` intent records; not a
new intent record. Base: `skrynia-tools` T1 exact `b5c9fe05`. Kernel target:
K4 `b8eff729` (the integrated `d81f16b` RAM resize loses retained-byte writes
and CAS, so it is not a qualification base). No kernel change, no production,
no old-ABI qualification as target acceptance.

## Guest surface

The compiled `skrynia_native_service.wasm` has exactly one import,
`p0.invoke(ctx: i32, op: i32, value: i32) -> i64`, and exports
`_start () -> ()`. There are no `skrynia.*` imports: no `read`, `write`,
`eval`, `eval_buffered`, `eval_delegate`, `fs_delegate_scope` or callback
function-pointer import. `ctx` is an opaque evaluator handle, never a RAM
address or a linear-memory pointer. P1 keeps private linear memory for its own
computation; nothing crosses the evaluator boundary through it.

`native-service/integration/verify_target_wasm.py` checks this surface on the
actual built artifact.

## Mediated userland byte protocol

`GET=0` reads one inbound byte (`0..255`, `-1` EOF/none), `PUT=1` appends one
outbound byte, `FLUSH=2` commits the framed generic kernel operation and
switches the stream to the reply, `CB_GET=3`/`CB_PUT=4` carry length-prefixed
callback frames, `STATUS=5` reports idle/active/trapped state. All values are
scalars; no private-memory pointer and no function-table index is passed.
A failed or uncertain FLUSH is final: the guest never replays a write.

The framed payload uses the generic wire opcodes already shared with the
proxy side (`OPEN_DISK`, `READ`, `WRITE`, `CLOSE`, `MKDIR`, `DELETE`, `RENAME`,
`OPEN_DIRECTORY`, `NEXT_ENTRY`, `SYNC`, `RANDOM_FILL`, `TRY_LOCK`,
`RELEASE_LOCK`, `EXISTS`, `EVAL`). P0 stages path/code bytes in its own
software-managed shared RAM inside a per-invocation slot; it stays a dumb
generic proxy and performs no token-verdict or app-policy operation for P1.
Locks remain userland CAS locks, never the removed kernel mutation lock.

Application wire protocol, authority checks, durable registry, CAS revision
activation, exact-key Web Push state, grants and wake hints are unchanged:
only the capability transport under `cb_guest/` was replaced.

## Nested P2 boundary and honest blocker

K2's eval child may import only one function, so P1 cannot import both the
supplied `invoke` and kernel `eval`. P0 therefore evaluates staged P2 code
with its own `invoke` function under a nested context; P2 exports `_start`
and imports only `p0.invoke`.

A P2 that needs P1-owned appdata callbacks cannot run on this P0-mediated
path: P1 is suspended while P0 evaluates P2, so a CB queue cannot reach the
suspended P1. On `STATUS=ACTIVE` the guest fails closed with `ENOTSUP (-4)`
instead of pretending success or deadlocking. Callback-dependent P2 guests
require the separate in-process userland interpreter owned by T6, which calls
the `Eval` trait callback closure directly inside P1's own store. T3 will
swap that evaluator into `MediatorEval` only after a real compatible artifact
with exact head and wasm32 build is delivered. Live end-to-end P2 callback
acceptance is therefore BLOCKED, not inferred from host tests.

## Validation boundary

Host tests exercise the exact framing with a deterministic fake evaluator;
the built WASM is checked structurally. These are scoped implementation gates,
not live target integration or release acceptance. Live integration additionally
needs T2's final P0 head, T6's interpreter artifact, A2's P2 artifact and a
disposable real-kernel harness at K4.
