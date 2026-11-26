# Program 0 on the target ABI

Derivative design for owner head `0ccc90f` (intent records) and K0 choices, not
a new intent record. T2 scope is `program0/` only: a token verifier and a dumb
generic byte-proxy. No application policy, no kernel change, no production.

## Entry and boundaries

The kernel still calls the root export `skrynia_main(input_fd, output_fd) -> i32`
(root validation requires it and an exported memory). Every ABI pointer and
length passed to `skrynia.*` is a `u64` offset in the single shared RAM; no WASM
linear-memory pointer crosses the boundary. Children are generic modules with
an exported `_start () -> ()` and at most one integer-signature import.

P0 stages the operator-selected child from `programs/root.wasm` into a disjoint
shared-RAM slot using byte-only `read1`/`write1`, then calls exactly one
`eval(code, size, invoke_selector, input_handle, output_handle)`. The child's
single import must be `p0.invoke(ctx: i32, op: i32, value: i32) -> i64`; P0
selects it from its own exported indirect table. `ctx` and the two eval handles
are opaque, never RAM addresses or function indices.

## invoke protocol (coordinated with T3 before implementation)

Coordinated on board issues `vQUJZ6fZ0` (T2) and `rNTsH6jffVX9qzKbF3hf8B`
(T3); T3 confirmed the signature, selector, entrypoint and import pair
`p0`/`invoke`, and refined the op codes:

| op | name | meaning |
|---|---|---|
| 0 | GET | next inbound/reply byte (`0..255`), `-1` at EOF/none |
| 1 | PUT | append one byte to the framed descriptor request |
| 2 | FLUSH | execute the framed request; reply bytes become readable via GET |
| 3 | CB_GET | next nested-guest callback byte, `-1` if none |
| 4 | CB_PUT | one byte to the nested-guest callback answer queue |
| 5 | STATUS | 0 idle, 1 nested guest active, 2 trapped, negative error |

Framed requests use the existing userland i32-LE opcodes shared with T3
(`1` OPEN_DISK, `2` READ, `3` WRITE, `4` CLOSE, `11` RANDOM_FILL, `12`/`13`
userland CAS lock, `14` EXISTS, `15` EVAL). Descriptor handles `0`/`1` denote
the per-invocation input/output streams. The reply is `[i32 code][data]`.

## Per-request isolation and recovery

Each HTTP request gets a fresh root instance and a fresh software context.
The shared-RAM allocator grows once to a fixed arena (monotone `truncate`,
which requires the K4 stable-page RAM fix) and claims disjoint 2 MiB slots by
a CAS byte per slot. There is no global request-control page, no native mutex,
no execution budget and no native lock/limit. Path strings and code bytes are
staged inside the claimed slot, so concurrent requests cannot overwrite one
another's scratch. A child trap returns a negative eval status; P0 does not
replay, CAS-frees the slot, closes descriptors and lets the next request
proceed. Lock ops are P0 userland CAS on a slot-local byte.

## Explicit blockers and non-goals

- `copy` four-argument semantics remain owner-unresolved; no `copy` import or
  guessed signature is generated. The byte pump is internal.
- Callback-dependent nested P2 guests are BLOCKED: a synchronous P1 -> P0.eval
  relay cannot service P2 callbacks while P1 is suspended. CB_GET/CB_PUT are
  byte queues only; no blocking relay is claimed. No-callback nested EVAL is
  relayed, but shares the parent stream handles and is not a full continuations
  design.
- Framed opcodes 5-10 (MKDIR/DELETE/RENAME/OPEN_DIRECTORY/NEXT_ENTRY/SYNC)
  return `-4` ENOTSUP in this increment; they are not claimed as ported.
- d81f16b is documented NO-GO for RAM-resize correctness; validation uses the
  K4 fix branch `b8eff72`. Old-ABI qualification is historical, not acceptance.
