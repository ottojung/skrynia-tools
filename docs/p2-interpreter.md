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

## Guest boundary (frozen with T2/T3/A2)

- Exactly one import: `p0.invoke(ctx:i32, op:i32, value:i32) -> i64`,
  selector 0. `ctx` is an opaque per-invocation handle, never a RAM address,
  memory pointer or function-table index.
- Exported `_start () -> ()` is the entry. A module start section, if
  present, runs at instantiation; exporting `_start` is still required.
- No `skrynia.*` import, no linear-memory pointer crossing, no second
  import. Validation is fail-closed before instantiation.
- Ops: `GET=0` next input byte (`-1` EOF), `PUT=1` append response byte,
  `FLUSH=2` accepted no-op, `CB_GET=3` submit one request byte to P1 and
  receive the first reply byte, `CB_PUT=4` drain the next reply byte
  (`-1` when drained), `STATUS=5` returns zero.
- Scalar error codes mirror the SDK: `-2` bad context, `-3` invalid value,
  `-4` unsupported (no callback capability), `-5` reply still pending,
  `-6` output cap reached.

## Failure modes and proof obligations

- A callback error is sticky. The guest may keep calling, but the P1 callback
  is invoked at most once per `CB_GET`; later calls return the same code and
  evaluation returns `Error::Callback(code)`. This is the no-replay rule for
  an uncertain reply.
- A new `CB_GET` is rejected with `-5` until the previous reply is fully
  drained, so a reply cannot be silently overwritten.
- Malformed modules, wrong import pair/signature, missing or wrong `_start`,
  link errors and guest traps produce distinct typed errors. No response is
  returned from a trapped or callback-failed evaluation.
- `Limits::output(cap)` bounds only the evaluator's own response buffer. It
  is a host memory guard, not an engine execution budget or resource
  metering, and is disabled by the kernel target rules.
- Each evaluation owns fresh input, output, reply and failure state; no
  global RAM page, native lock or cross-store handle is shared.

## Integration note

T3 owns `native-service/`. This component does not edit that crate; the
`Eval` implementation is provided here so T3 can adopt it by selecting this
capability. A2's `_start` + `p0.invoke` fixtures bind unchanged. No live
nested-kernel integration is claimed by this component alone.
