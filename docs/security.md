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

## Release integrity commitments

New code releases store an SHA-256 commitment to all copied artifact bytes, relative paths, directory entries and executable flags. `app verify` checks this explicitly, including over restricted SSH. It is not part of rollback and does not imply any consistency of application data, storage migrations, running processes or external hosting caches. A host operator with write access to both the metadata and release trees remains trusted.

## Administrative grant recovery

Optional grant provisioning writes a private journal before remote creation, saves the one-time issuer response before user issuance, and marks `issuing_user` on disk before sending that request. `admin inspect` reports recovery state without printing capabilities. A failed/uncertain user-token request may have minted an unrecorded token. For that state, `admin resume` refuses to reissue unless the operator explicitly passes `--accept-duplicate-token-risk true`. Reissuance cannot revoke an orphaned token or guarantee exactly-once semantics. A lost grant-creation issuer response is unrecoverable through this client; repeating `POST /grants` would create an additional grant, so the CLI refuses automatic recovery. Private journal changes serialize under a persistent owner-only filesystem lock.


## Explicit revocation and uncertain DELETE

`admin revoke-user` and `admin revoke-grant` use the locally held issuer capability in the private child-grant journal. The CLI requires the exact grant ID again before either destructive **authority** change. It persists an in-progress state before the HTTP DELETE and requires explicit acknowledgement to retry an uncertain result. A 404 is treated as confirmed absence only with the expected Skrynia `user_not_found` or `grant_not_found` error code; an arbitrary 404 does not confirm revocation. Once done, corresponding credentials are removed from the journal; `admin inspect` never reveals them. Root credentials are not a supported journal input. A revoked capability can still exist as a copied secret elsewhere but must be rejected by Skrynia if revocation was committed.

Grant revocation does **not** erase stored bytes; do not confuse authority lifecycle with data retention or application release lifecycle. Deleting a child grant also affects subordinate users/grants, so treat `revoke-grant` as a privileged operation.

## Build timeout containment

Git/build tool subprocesses are started in separate Unix process groups; timeout cleanup signals their group and reaps the direct process. Captured output is bounded and concurrently drained. Podman build failures additionally trigger container removal. A script that deliberately daemonizes into a different session, starts a remote process or escapes the configured container boundary may outlive this group, so this is failure containment rather than a universal process-sandbox guarantee.

## Podman builder-image boundary

The builder-image selector is strictly one validated OCI-style reference. The runner inserts an explicit \`--\` argument terminator before the image, disallowing option interpretation (such as \`--privileged\`, \`--network\` or \`--mount\`). The builder command remains fixed to \`make build\`, in rootless Podman with all Linux capabilities dropped and no operator credentials mounted. Image validation does not establish that the selected image's contents are trustworthy; pin reviewed images by digest for reproducibility.

## Remote CLI input and cancellation

Remote SSH management uses an explicit-size JSON message (at most 16 KiB) and an overall command deadline, including pipe writes. The local SSH process runs in an isolated process group; a blocked stdin pipe cannot indefinitely bypass timeout supervision. Local cancellation may leave a completed or still-running remote deployment, so caller-facing errors identify uncertain effects rather than claiming rollback. Inspect the remote active release before deciding to resend.

## Browser URL and credential boundaries

The browser client rejects non-HTTP(S) base URLs, embedded usernames/passwords, fragments and queries. This matches the native client's URL policy and avoids treating credentials embedded in a URL as a supported authentication workflow. Browser tokens are supplied explicitly as optional capability strings and sent only on authenticated operations; public GETs remain unauthenticated. Browser JavaScript cannot hide tokens bundled in public code, and browser cross-origin requests still require appropriate server CORS headers.

## Exact HTTP commitment and read-response statuses

The native and browser clients do not infer that *every* 2xx status means an operation completed. Skrynia's versioned contract returns **201 Created** for object creation and **200 OK** for confirmed replacement/deletion and grant operations; object GET and list results must also be **200 OK**. `409` is the create collision, `412` the CAS mismatch, and `404` the missing object. Unexpected successful responses (e.g. `202 Accepted`, `204 No Content`, or `206 Partial Content`) are reported as contract violations rather than silently treated as committed writes or complete reads. A mutation receiving an unexpected success status may still have taken effect, so the caller must reconcile it with an authoritative read before retrying.

## Bounded API metadata and diagnostics

Native client error responses are read with a 4 KiB cap, rather than materializing an arbitrarily large error body. Expected JSON responses (pagination metadata and grant issuance/revocation) have a 1 MiB maximum, enforced both through Content-Length and through a streaming read bound for responses with unknown length. Overlarge responses are explicit errors. **Raw GET object payloads are intentionally not subject to the JSON cap**, since applications can store arbitrary byte objects; applications needing stricter object-size budgets should implement those at their own data protocol boundary. Browser fetch still follows its own memory model and must not be mistaken for a bounded streaming client.

## Isolated real-server credential lifecycle regression

The source-pinned Skrynia CI compatibility job now also exercises `admin create`, `revoke-user`, `revoke-grant` and repeated completed revocation against a disposable localhost server and grant hierarchy. It confirms a revoked user is rejected when attempting a new object creation, a revoked grant's former issuer cannot issue a user, the journal has shed its secrets, and the bytes published before revocation remain readable. All test credentials are private under a temporary owner-only directory, never reused from a production deployment. This checks server/client interoperability but not physical power-loss recovery or complete credential-lifecycle security.

## GitHub Actions result attestation boundary

The composite action confirms the remote release metadata's name, exact repository URL, source subdirectory, builder image and any requested full Git commit **before** writing GitHub Action outputs. It also enforces a bounded SSH session (35 minutes) rather than hanging indefinitely if the remote connection stalls. This is metadata consistency checking, **not cryptographic host attestation**: the authorized deployment host controls its response and is trusted to build and publish the supplied source. An SSH timeout cannot roll back changes already accepted by the server. When an outcome is uncertain, reconcile by inspecting the remote release before resubmitting.

## Durable activation generations and ABA protection

Application activation/rollback has a per-application eight-byte activation generation saved under the private \`.locks\` directory. The generation is atomically persisted and the lock directory synchronized **before** a release symlink is replaced, so a build launched under A cannot unknowingly override a later A → B → A administrative sequence. A build observes both the active release and its generation under the short activation lock, and rechecks both before switching after its long build. Counter changes are O(1) and independent of release history and artifact size. Failed/uncertain activations can conservatively advance the counter even when the symlink remains unchanged. This is a protocol for **cooperating CLI activations** and relies on durable filesystem ordering; manual symlink edits, hostile host owners, and filesystems that violate fsync ordering are outside the guarantee. Corrupted/symlinked generation files reject activation rather than resetting the counter. The read-only app audit reports such corruption as generation_error and returns nonzero, including when release artifacts themselves remain intact.

## Auditing abandoned code-publication intermediates

The read-only app audit also detects hidden candidate symlinks from interrupted atomic release activations and app-specific build workspaces that survived a crash or failed cleanup. It reports both without deleting them. A running deployment may legitimately expose these intermediate entries, so inspect again when the build lock is released before diagnosing a persistent leak. Other applications' workspaces and private files are outside a per-app audit.
