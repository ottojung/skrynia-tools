# Security and failure boundaries

**Status:** the native HTTP client, CLI, CAS cell, append log and static release deployment are implemented and smoke-tested. The additional fault-injection, adversarial verification and WebAssembly guarantees described here are requirements for future production qualification, not claims about tests already completed.

Skrynia enforces one-key atomic operations and scoped bearer grants. It does not enforce client protocols, guarantee execution by other clients, or provide private reads. Library users must distinguish storage-enforced authority, cooperative assumptions, and independently checked payload validity.

## Credential model

- **Issuer tokens** issue/revoke exact-grant user tokens and revoke the grant subtree; **user tokens** perform operations, including `delegate` when granted.
- Never put root or issuer credentials into public bytes, JavaScript bundles, URLs, logs, Git or ordinary CLI JSON. Admin secrets are stored in owner-only local/private files, written atomically, and never emitted without explicit private export.
- A public browser `list` token intentionally exposes enumeration within its matching scope. A browser publisher token can be extracted by its holder; grant narrowing, not JS secrecy, defines the protection.
- Grant authority is decided by Skrynia on every operation; locally stated scope is not proof of authorization.

## Unknown outcomes and recovery

A request can commit even when the response never arrives. Use operation/transaction IDs and authoritative reads to resolve ambiguous append or CAS outcomes when possible. Otherwise report an unresolved/unknown outcome, not an invented failure or success. The native HTTP client now exposes `ClientError::OutcomeUnknown` after failed mutation transport, `ClientError::Transport` for failed reads, and `ClientError::Rejected` with Skrynia response status and error code. Recovery of a lost grant-create issuer response remains impossible without server-side idempotency. Avoid invisible unbounded retries.

Grant creation is especially sensitive: losing its one-time issuer response may create an orphan authority that a parent cannot automatically recover or revoke individually. Provisioning is a staged, journaled **non-transactional** operation. Durable receipt of each secret must precede the next remote operation; stop on unreconciled ambiguity.

Revoking a grant never deletes stored object bytes. Data deletion and credential revocation are distinct, destructive, explicitly authorized workflows.

## Transport and protocol assumptions

- Read mutable CAS heads authoritatively. CDN/static server cache validators may not match byte-SHA256 CAS semantics. Verify bytes where necessary.
- Untrusted clients may create malformed log entries or fill future create-only slots; create-only prevents them from modifying existing entries, **not** from disrupting availability.
- Listings may require multiple non-snapshot pages; Web Push is a coalesced `changed` hint rather than an event stream.
- Validate keys, matchers, bounded payload sizes and protocol versions at input boundaries. Expose conflict, quota exhaustion, insufficient authority, cancellation and network uncertainty explicitly.
- Confidentiality requires a separate client-side cryptographic and key-distribution protocol; Skrynia GET remains public.

Independent publishers from mutually untrusted groups need separate HTTPS browser origins. Do not claim that an SDK eliminates same-origin or bearer-token security boundaries.

## Local publishing controls

Private issuer-token files use owner-only permissions, refuse symlinks and are fsynced along with their parent directory before provisioning continues. Credential writes atomically replace a private file before issuing the next credential. Rootless Podman runs builds without adding the operator's Skrynia credentials or SSH key material to the container. Static releases are copied and synchronized separately from source code. The CLI is **not** intended to accept arbitrary untrusted callers on a shared host: users with deployment-host filesystem access can alter release metadata or files.
