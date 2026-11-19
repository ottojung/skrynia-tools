# Optional Program 0 for Skrynia v3

Skrynia is **only the VM server** and does not bundle any Program 0.
This repository supplies **one optional** user-controlled WASM implementation
under `program0/`, compiled independently and installed by the operator.
An operator may instead supply any other conforming Program 0 module.

The VM loads a module from `SKRYNIA_ROOT_WASM` on every initialization or
restart. The module exports:

```text
skrynia_main(input_fd: i32, output_fd: i32) -> i32
```

Both arguments are opaque, forward-only stream descriptors, **not** WASM
memory addresses. The module reads the first using the privileged
`skrynia.read(input_fd, dst_private_ptr, len)` import until EOF, then
writes its output with `skrynia.write(output_fd, src_private_ptr, len)`
in a loop to handle short writes. It may close either descriptor. The
input forbids writes; the output forbids reads. Neither can seek, truncate
or sync. Program
0 has its own private linear memory, wholly separate from global RAM.
Evaluated child programs do **not** inherit Program 0 privileges.

## Build

Use Rust 1.93 and the `wasm32-unknown-unknown` Rust target:

```sh
cargo build --locked --release -p skrynia-program0 --target wasm32-unknown-unknown
```

The output is `target/wasm32-unknown-unknown/release/skrynia_program0.wasm`.
With Rust source installed but no target stdlib, the equivalent development
cross-build is `RUSTC_BOOTSTRAP=1 cargo -Z build-std=std,panic_abort build
--target wasm32-unknown-unknown --release -p skrynia-program0`.

Point a **disposable Skrynia v3** instance's `SKRYNIA_ROOT_WASM` at this file,
and provide a separate, confined data directory. A release must not point at
an existing v2 data tree until migration and rollback are validated.

The current example program implements a small, application-defined
register/set/invoke/lookup/revoke protocol in WASM. It is **not yet a complete
implementation of the historical v2 grant-based wire protocol**. That parity
belongs to this repository's compatibility facade and is a release gate.
No CLI is required or supported for Program 0.


## Executable cross-repository integration

With both repositories built independently, verify the actual HTTP behavior
using the explicitly ignored test (it never touches production data):

```sh
SKRYNIA_V3_SERVER_BIN=/absolute/path/to/skrynia-server \
SKRYNIA_V3_PROGRAM0_WASM=/absolute/path/to/skrynia_program0.wasm \
cargo test --locked -p skrynia-tools --test v3_live -- --ignored --nocapture
```

The harness creates a fresh temporary storage directory, launches and
restarts the external server process, then verifies register/invoke/set/
restart/invoke/revoke through real `POST /` requests. Missing env variables
fail explicitly when this opt-in test is run; ordinary unit test executions
report it as ignored, not as a false parity pass. This tests the small
example registry only, not the complete historical grant protocol.

## Targeted program-record operations

The optional Program 0 handles its binary registry requests directly
against the injected disk capability. Each register operation probes the
new random candidate ID and refuses a collision; lookup, invoke, set and
revoke read the record for exactly the requested ID. The root no longer
lists every program and loads every module on every HTTP request. Its
request-time program lookup work scales with the selected code record,
not with the total number of registered programs. A memory-backed test
capability deliberately fails if directory enumeration is attempted.

This improvement does not solve concurrent writers to a program ID:
the kernel disk API guarantees atomic same-filesystem rename, not an
atomic check-and-rename of a stored program record. Authorization and
cross-request transaction isolation remain application-protocol duties.
Likewise, the current optional root still materializes the selected
module and its result in private WASM memory before emitting output.

## Private initialization of the provisional v2 authority

The optional root's preliminary v2 storage authority is initialized lazily
when its first v2 mutation arrives. Program 0 generates 256 random bits
through its explicit secure-random capability, persists them under the
private data path v2auth/root-secret, then persists an authority containing
only the token hash. The incoming request NEVER sets or receives this token.
The public V2Bootstrap binary operation is unconditionally rejected.

If the process stops between writing the private token and the authority,
the next request reuses the private token and completes initialization;
malformed or inconsistent pre-existing private state fails closed.
The initial request still needs the independently obtained private token
to authorize mutations. The kernel creates new files owner-only (0600),
and the operator must restrict the entire data root to owner access (0700).
Do not serve v2auth through a static web root, expose this token to browsers,
or place it in public objects, logs or deployment manifests.

This is ONLY a provisional root-level create/modify/delete/CAS bridge:
it does not yet implement the v2 grant-tree issuance, scoped quotas,
subscriptions, list or complete HTTP facade.

Stored provisional v2 object records carry their original logical key as
well as bytes. Reads reject corrupt JSON and key/hash mismatches as an
uncertain-state error rather than reporting the object missing; neither
create nor replace may silently overwrite damaged data. The byte encoding
of an object is never used as the CAS validator: the actual logical bytes
are SHA-256 hashed for comparison, as in v2.

For provisional v2 mutation requests, logical keys use the same 255-byte
ASCII grammar as the native v2 server: nonempty slash-delimited components,
letters, numbers, underscores and hyphens, with dots permitted only after
the first character of each component. Invalid, empty or oversized keys
return invalid_key before any mutation; stored paths remain SHA-256-derived.
