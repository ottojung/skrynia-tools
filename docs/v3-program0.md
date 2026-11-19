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

The current example Program 0 implements a native, application-defined
binary POST protocol: code registration/invocation and scoped app data
read/write with separate capability secrets and conditional CAS. It does not
emulate the withdrawn older HTTP GET/store/grants/Web Push surface. Operator
code activation authorization, issuance and bounded registration require
separate release qualification. This Program 0 now uses a kernel-owned
request-scoped mutation lock instead of a non-expiring shared-RAM CAS bit. There is no CLI.

### Native mutation lock ABI

This optional Program 0 requires native host imports
try_mutation_lock() -> i32 (1 acquired, 0 busy) and
release_mutation_lock() -> i32 (0 released, -4 nonowner).
It holds this kernel-owned lock for code registry and appdata mutations.
The kernel automatically releases the lock when the root VM terminates,
including fuel exhaustion and WASM traps. This import addition requires
upgrading the kernel and Program 0 together. General cas_ram_u64 remains
available for shared RAM but is no longer used for this transaction lock.
Program 0 still must sync/rename staged disk writes before publishing and
reconcile ambiguous client responses after a crash.

Appdata object keys are nonempty byte strings of at most 256 bytes, scoped
under a 32-byte read capability. The disk stores a fixed 64-hex SHA-256 digest
of each raw key plus the four-byte .bin extension as a filename (68 bytes),
rather than hex-encoding raw input bytes into an unbounded path component.
This avoids filesystem NAME_MAX failures even for valid 256-byte keys.
Separate read capabilities isolate namespaces. This is a new native layout,
not a compatibility layer: older developmental appdata snapshots with raw
hexadecimal filenames must be explicitly exported and migrated by a trusted
operator before upgrading; no silent rewrite or automatic key discovery.


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

## Native immutable code releases and activation (draft, v3 ABI version 1)

Program 0 defines new **binary POST** operations for version management.
They are NOT HTTP GET/PUT/store/grants compatibility operations. All
identifiers and code digests are 32 raw bytes; an id field carries a little
endian 16-bit length followed by those bytes, and a blob field carries a
little endian 32-bit length followed by the bytes.

- Opcode 9, StageCode: admin_cap id, WASM code blob. Stores an immutable
  content-hash blob and returns its 32-byte SHA-256. Requires a live app
  already owned by that private admin capability. Unsealed blobs cannot
  be activated.
- Opcode 10, SealCode: admin_cap id, 32-byte code hash. Re-reads and
  verifies the complete staged blob and fsyncs a per-app immutable
  readiness marker. This operation costs O(code bytes) only at release
  preparation; readiness is an explicit operator assertion, **not** proof
  of kernel execution safety, test coverage, or authorization of child
  operations. Use a disposable native VM to validate before sealing.
- Opcode 11, ActivateCode: admin_cap id, expected_revision u64 LE,
  target_ready_hash 32 bytes. Under the exclusive native mutation lock,
  checks an exact revision and the fixed-size ready marker, then atomically
  writes a fixed **42-byte** code pointer and advances its revision.
  Returns the new 42-byte active pointer. No historical list scan, code
  read, rehash, copy or migration on activation/rollback. Repeating a
  successful but acknowledgement-lost activation with the stale revision
  returns a conflict. Read current state to reconcile.
- Opcode 12, GetActive: invoke_id id, returns the 42-byte current
  pointer. The pointer format is v3: [3, live_byte, 32-byte hash,
  u64_le_revision]; tombstones remain durable with advanced revisions.
  A re-register after revoke **never** resets the revision, so a stale
  A->B->A request cannot become valid after a revoke/recreate.
- Opcode 13, ListVersions: invoke_id id, cursor (one byte 0/1 plus
  optional 32-byte last returned hash), limit u16 LE (1..256). Returns
  u16 LE count followed by that many ascending 32-byte ready hashes.
  List and paging may cost O(number of historical releases) in this
  implementation; the **activation** operation never pays this cost.
- Opcode 14, ListPrograms: cursor and limit in the same format;
  returns sorted public 32-byte invoke IDs in pages of 256 or fewer.
  These APIs expose public IDs and version hashes but never grant
  mutation authority. Mutations require the operator's private
  32-byte admin_cap, not its SHA-256 public invocation identifier.

The root's active record is an **incompatible new native v3 format**.
Prior experimental v2-format active files and hex-encoded object basenames
require explicit trusted export/import; no automatic legacy translation.
The per-app release marker is written only after its full code blob has
been durably published and cryptographically verified. Blob reads also
verify the actual bytes match their content hash; corrupt records fail
closed. The global historical blob store may retain unused versions;
garbage collection and quotas are separate security/operational tasks.

Important release blockers: Register remains an unqualified anonymous
user-secret-based operation. It can allocate unbounded durable code
storage and must be protected by operator-owned installation authority
and quotas before enabling public internet access. The current code
readiness marker represents the trusted admin's preparation, not
independent WASM compile approval. Production migrations, Web admin
integration, browser credential issuance, aborted-write recovery and
security review remain incomplete.
