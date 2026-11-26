# Ordinary P1 native service: target ABI

This is implementation documentation, not an owner intent record. P1 is an
ordinary userland program, not part of the Skrynia kernel.

## Guest surface

Build the P1 artifact with the `cb-guest` feature:

```sh
RUSTFLAGS='-C link-arg=--export-table -C opt-level=z -C strip=symbols' cargo build --locked --release \
  --target wasm32-unknown-unknown -p skrynia-native-service --features cb-guest
```

The artifact imports exactly one function,
`p0.invoke(ctx: i32, op: i32, value: i32) -> i64`, and exports
`_start () -> ()`. It has no direct kernel imports. Private linear memory
is for P1 computation only; no private pointer crosses the ABI boundary.
Use `native-service/integration/verify_target_wasm.py` on the built artifact.

## Outer P0 byte protocol

The P0 gateway owns the authenticated request stream and mediates generic
descriptor and RAM operations. GET (0) reads one inbound byte; PUT (1)
appends one byte to a framed operation; FLUSH (2) executes the complete
frame and exposes its reply to GET. Drain the complete framed reply,
including the terminating GET result -1, before sending another frame
or returning to input. The reply EOF does not consume an input byte.
Check every PUT and FLUSH status and the operation result. STATUS (5)
reports state; outer callback-queue operations are unsupported.

The application response is a framed WRITE (opcode 3, output handle 1,
u32 byte length, payload), not a raw PUT stream. An uncertain write is
never automatically replayed. P0 stages paths and code in software-managed
shared RAM. It implements generic mediation, not application policy.

## Nested P2

P1 executes stored P2 modules with its own Wasmi interpreter, using the
single-function protocol in [p2-interpreter.md](p2-interpreter.md).
P2 callback frames are executed synchronously on P1's own stack, with
the existing Registry::invoke appdata capability; their complete original
request bytes are passed to P1's authority checks. This avoids deadlock
through a suspended P1 or an extra kernel import. A failed guest evaluation
returns an application error rather than partial successful output.

The P2 protocol is deliberately separate from the outer P0 mediation
protocol. Application request/response bytes, namespace checks, revision
CAS, durable registry and WebPush behavior are maintained by P1.

## Qualification

Use exact, freshly built kernel, P0 and P1 artifacts for transport tests.
A host interpreter test or host-preseeded registry fixture alone does
not qualify genuine registration, mutation, restart persistence,
concurrent operations or Theorem Lab application deployment. Historical
proofs against earlier commits are not current integration evidence.
