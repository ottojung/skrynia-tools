# Trusted Program 1 scoped capability ABI (proposed)

Status: **proposal / kernel extension required**.
No production cutover. Do not claim durable storage works until a real
kernel implements and independently verifies this boundary.

## Goal

Keep Program 0 a dumb byte proxy to one fixed operator-owned guest.
Move native application protocol, persistent appdata, CAS, program
registry, sealed immutable ready versions, O(1) activation/rollback
with monotonic revision (ABA-safe), authenticated operator/admin flows
and restart safety into an operator-installed **trusted Program 1**
service. Untrusted application Wasm must remain isolated.

## Threat model and non-goals

- Program 0 receives no application semantics and forwards opaque bytes.
- Only an operator-selected trusted guest may receive scoped privileges.
- Arbitrary user-controlled code is never treated as trusted.
- No blanket `open_disk` or global RAM access for untrusted app Wasm.
- No v2 HTTP GET/HEAD/store/grant compatibility facade and no CLI.
- Capabilities are least-authority, auditable, and namespace-confined.

## Minimal required kernel additions

The current `skrynia.eval` guest ABI buffers input/output in linear
memory, passes an optional callback table index, and lets guests import
`skrynia.invoke` only in the proxy-isolated case. A trusted Program 1
service needs persistent storage and operator-controlled privileged
operations without exposing those imports to untrusted guests.

### 1. Trusted guest selection

Kernel must instantiate the trusted service guest with a **privileged
import set** only when all hold:

- The module is the operator-configured fixed guest for Program 0
  forwarding (e.g. `programs/root.wasm` hash pinned by operator policy),
  or an explicitly operator-registered trusted Program 1 module.
- The operator has enabled trusted mode for that exact module hash/id.
- Untrusted app modules evaluated by the service never receive the same
  import set.

Selection must be by kernel/operator policy, never by guest request.

### 2. Scoped disk capability (required for durable state)

Expose disk imports only to the trusted guest, confined to a
kernel-assigned service root (for example `data/program1/`):

- `td_open(path_ptr, path_len, mode) -> fd`
- `td_read(fd, dst, len) -> n`
- `td_write(fd, src, len) -> n`
- `td_close(fd) -> rc`
- `td_mkdir(path_ptr, path_len) -> rc`
- `td_exists(path_ptr, path_len) -> rc`
- `td_delete(path_ptr, path_len) -> rc`
- `td_rename(old_ptr, old_len, new_ptr, new_len) -> rc`
- `td_truncate(fd, size) -> rc`
- `td_seek(fd, off) -> rc`
- `td_sync(fd) -> rc`
- `td_open_dir(path_ptr, path_len) -> fd`
- `td_next_entry(dir, dst, cap) -> n`
- `td_is_dir(path_ptr, path_len) -> rc`

Rules:
- Paths must be normalized and rejected if they escape the assigned
  namespace (`..`, absolute, symlink traversal, NUL, empty segment).
- No access to `programs/operator.cap`, other tenants, or host paths.
- Atomic write primitive or rename-over-fsync semantics required for
  CAS-ready records and active-pointer updates.
- Directory listing must be bounded and deterministic per syscall.

### 3. Scoped operator secret access

- `td_read_operator_cap(dst, cap) -> n` reads only the operator
  registration capability file configured for this service, not
  arbitrary files. Kernel enforces read-only access and fixed maximum.

### 4. Mutation serialization / restart safety

Provide either:
- `td_try_mutation_lock() -> rc` / `td_release_mutation_lock() -> rc`
  scoped to the trusted service namespace, or
- a kernel guarantee that service eval/invocation is serialized.

The service must be crash-safe: partial writes must not be observed as
committed active/ready state; restart must rebuild transient caches only
from validated durable records.

### 5. Randomness and time (if required)

- `td_secure_random(dst, bits) -> rc` for capability/nonce generation.
- `td_now() -> i64` only if protocol requires it; avoid ambient time in
  deterministic state transitions where possible.

### 6. App evaluation callback (existing, must remain mediated)

Trusted service may evaluate stored app Wasm with mediated callbacks:
- `skrynia.eval(...)` / callback dispatch only reaches service-defined
  handlers.
- Callbacks must enforce per-object read vs write capabilities and
  namespace isolation (`SelfNs` vs external caps).
- Evaluated untrusted app Wasm receives no direct disk/RAM/FD imports.

### 7. Streaming follow-up (separate kernel work)

Current eval ABI is memory-buffered (proxy caps: 16 MiB input, 8 MiB
code, 8 MiB output). True FD-streaming trusted guest execution requires
a separate safe ABI; do not weaken isolation to achieve streaming.

## Capability delegation model (service-level)

- **Operator cap**: authorizes application registration/install actions.
- **Admin cap**: per-application secret; derives invoke ID / namespace.
- **Read cap**: derived/public read capability for appdata objects.
- **Write cap**: secret write capability; separate from read cap.
- **Invoke/App callbacks**: mediated, scoped to presented capability and
  target namespace; no ambient authority.
- **Releases**: immutable sealed blobs + ready markers; activation is a
  fixed-size CAS on active record with monotonic revision.

## Verification requirements before claiming readiness

- Real kernel + WASM + browser/protocol tests, not only unit tests.
- Crash/lock safety and restart recovery.
- Malicious guest attempts: `open_disk`, path escape, capability forgery,
  stale CAS/ABA replay, unsealed activation, wrong-namespace callback.
- Data export/import path for migration (explicit, not implicit).
- Secure cap issuance and A-B-A release history checks.

## Current implementation status

- Program 0 proxy: implemented as dumb forwarder (see `program0/`).
- `native-service/` library: preserves tested algorithms but is not a
  runnable/evaluated unprivileged guest and has no safe kernel callback
  ABI in this repository state.
- Trusted Program 1 guest with durable kernel-backed storage: **not yet
  implementable without kernel ABI above**. Stubs must fail closed and
  must not leak privileges.
