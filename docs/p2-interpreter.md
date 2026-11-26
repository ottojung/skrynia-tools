# Userland P2 interpreter (target T6)

Scope: a reusable, capability-injected component for running a nested P2
WASM guest inside P1. It is a library only: no CLI, no kernel change, no P0
gateway edit, no app policy, no production cutover. The interpreter is
`wasmi` 2.0.3, a mature pure-Rust WASM engine; it is not a hand-written
instruction subset with a general-compatibility claim.

## Contract

`Interpreter` implements `skrynia_native_service::cap::Eval` and also exposes
the same shape directly:

```
evaluate(code: &[u8], input: &[u8],
         callback: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>)
    -> Result<Vec<u8>, Error>
```

The callback is P1's existing appdata capability (`Registry::invoke`'s
`appdata::cb::guest_callback`). It is invoked on the evaluator's own stack,
inside P1. There is no P0 `CB_GET`/`CB_PUT` queue, no reentrant `P0.invoke`,
no suspended P1 continuation, no new kernel import and no implicit authority.
This is the independent P1 execution path required by the owner 17:36Z
warning: a suspended P1 cannot service a callback through P0.

## Guest boundary

The P2 userland protocol is distinct from the outer P0 gateway. P2 imports
only `p0.invoke(ctx: i32, op: i32, value: i32) -> i64`, with context 1.
It exports `_start () -> ()`; a module start section, if present, runs during
instantiation. No kernel imports or cross-store pointers are supplied.

| Op | Name | Meaning |
|---:|---|---|
| 0 | GET | Read next input byte, or -1 at EOF |
| 1 | PUT | Append one response byte |
| 2 | FLUSH | Accepted no-op |
| 3 | CB_GET | Read next callback reply byte, or -1 at EOF |
| 4 | CB_PUT | Append one opaque request byte |
| 5 | STATUS | Ready (0) |
| 6 | CB_FLUSH | Execute the complete pending callback frame once |

CB_FLUSH invokes the P1-owned callback synchronously with the entire frame.
The callback receives unchanged application request bytes, including its
authorization fields, keys, values and preconditions. Registry authorization
checks remain responsible for restricting AppGet/AppPut/AppDelete to the
invoked namespace. The evaluator does not hardcode keys or translate
application operations. A successful CB_FLUSH returns 0; then the guest
drains its reply before assembling another request.

## Failure and resource policy

- A callback or output-buffer failure is sticky. Subsequent gateway calls
  return the same error; no automatic replay or partial successful response.
- Empty flushes, unconsumed replies, oversized request/reply frames and
  unfinished request frames fail closed. A frame assembled without flushing
  has no callback effect.
- Error codes include -2 bad context, -3 invalid value, -4 unavailable
  capability, -5 framing error, -6 buffer limit.
- Each evaluation has its own store, input/output and callback frame state.
  Default *userland* limits: 100 million Wasmi fuel units, 64 MiB linear
  memory, one memory/table/instance, 1 Mi table elements, 1 MiB output and
  16 MiB callback frame/reply. These are provisional library policy, not a
  Skrynia kernel limit or a full accounting of callback-side resource usage.
  Fuel and callback cap can be overridden through Limits.
- Portable Wasmi dispatch is enabled explicitly, avoiding dependence on
  compiler tail-call optimization on native debug or wasm32 builds.

The interpreter's callback contract is tested with real WASM, including
multi-byte frames, failure without replay, limits, and a registered guest
performing appdata mutations with the actual Registry callback. Full
transport and application deployment qualification is separate.
