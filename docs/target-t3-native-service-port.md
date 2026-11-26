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

`GET=0` reads one inbound byte (`0..255`, `-1` EOF/none; T2's GET falls through
to the authenticated input descriptor), `PUT=1` appends one byte to the framed
generic operation, `FLUSH=2` executes that frame and exposes the reply to GET,
`CB_GET=3`/`CB_PUT=4` carry length-prefixed callback frames, `STATUS=5` reports
idle/active/trapped state. All values are scalars; no private-memory pointer
and no function-table index is passed. A failed or uncertain FLUSH is final:
the guest never replays a write.

The final application response is itself a framed WRITE (opcode 3, output
handle 1, `u32` byte length, payload, chunks of at most 64 KiB). Raw response
bytes are never PUT directly: the evaluator interprets every PUT frame as a
generic kernel operation. This is the framing implemented by T2 P0 head
`22c5db2e`.

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
supplied `invoke` and kernel `eval`. P0's opcode 15 stages code bytes only;
the child reads its input from the invocation descriptor. P0 therefore
evaluates staged P2 code with its own `invoke` function under a nested
context; P2 exports `_start` and imports only `p0.invoke`.

A P2 that needs P1-owned appdata callbacks cannot run on the P0-mediated path:
P1 is suspended while P0 evaluates P2, so a CB queue cannot reach the suspended
P1. The `MediatorEval` path therefore still fails closed with `ENOTSUP (-4)` on
`STATUS=ACTIVE` rather than pretending success or deadlocking.

## T6 in-process interpreter selection

The delivered T6 component (`skrynia-tools` head
`bac975b1610f6bec38ebe7078ccbdc1736078998`, `src/p2`, `docs/p2-interpreter.md`)
is reused verbatim: `native-service/src/p2` is a symlink to that source tree, so
no T6 file is edited. `cb_guest::eval::P1CallbackEval` selects
`p2::Interpreter` as the ordinary P1 `Eval` capability; it implements the
unchanged `cap::Eval` trait with `Err = i32`. `CB_GET/CB_PUT` call the callback
closure that `Registry::invoke` already supplies (`appdata::cb::guest_callback`)
directly on P1's own stack. The adapter frames the interpreter's single opaque
request byte into a complete `AppGet` request before handing it to that
callback, because the appdata capability decodes whole wire frames. There is no
P0 callback relay, no suspended P1, no privileged import and no cross-store
pointer. Sticky failure/no-replay and fail-closed guest validation are T6's
contract and are exercised by real-WASM host tests on the exact integrated
source.

## Validation boundary and disposable real-kernel proof

Host tests exercise the exact framing with deterministic fake capabilities;
T6's 12 real-WASM guest tests cover callback roundtrip, sticky no-replay,
wrong context, malformed/extra imports, missing/wrong `_start`, invalid bytes,
trap and response cap. The built WASM is checked structurally: exactly one
`p0.invoke` import, `_start` export, no `skrynia.*` import. These are scoped
implementation gates, not Theorem Lab end-to-end acceptance or a packaged P1
module claim.

A default release build that actually links `wasmi` is ~2.2 MiB and the kernel
rejects it before serving a request (compile child has a 4 s CPU and 64 MiB
precompile limit). `RUSTFLAGS='-C opt-level=z -C strip=symbols'` produces a
~1.5 MiB artifact that boots; the size-optimized build is required for any
real-kernel run.

The earlier P0 relay (`985d238b`) could not complete a nested appdata roundtrip:
its invoke context started its RAM cursor at zero while the real RAM fd sat
just past the staged guest, so staged paths were read from the wrong offset and
`open_disk` returned `E_IO (-5)` before the interpreter was entered. T2's
replacement P0 (`49a17d97`, WASM `bb3b8afb`) inherits the staging cursor,
resynchronises after nested EVAL and relays framed ops 5-10. With that P0 and
the size-optimized P1 (`e9a65426`), the disposable harness
`native-service/integration/nested/nested_cb.py` boots K4 `b8eff729`
(`c82ab502`) and passes all six checks: nested echo, P1-owned appdata
`CB_GET`/`CB_PUT` roundtrip (inner body `P1-OWNED-CB-ROUNDTRIP-OK`), sticky
callback failure, malformed-import fail-closed, unknown-op fail-closed
(`EINVAL -3`, no silent no-op) and wrong-bearer denial. The result is
`T3_T6_NESTED_CB_PASS`. Paths are rebound with `SKRYNIA_*_WASM` and
`SKRYNIA_NESTED_WORKDIR`; every artifact SHA is re-pinned in the harness.

This is a bounded disposable proof of source correctness and the nested
callback mechanism only. No live browser, provider, registered program,
Theorem Lab P2, P1/PX arm, deploy, merge, production cutover or old-ABI
qualification is claimed.
