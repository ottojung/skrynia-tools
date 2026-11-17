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

## HTTP application deployment trust boundary

Skrynia is the only server. Publishing happens on the caller's machine
or ephemeral CI runner through authenticated object POST and CAS PUT.
The deployment token is a scoped user credential, not an issuer token or
root credential. It must not be included in public build artifacts,
source history, public logs or the build container environment. The
GitHub Action builds the application before introducing the upload token.

An application release consists of ordinary hierarchical byte objects,
a JSON manifest with relative paths, SHA-256 digests and byte counts,
and a fixed-size ready marker. Assets are created before the ready
marker. The final active pointer is one Skrynia object, changed by
byte-SHA256 CAS. A concurrent activation produces a conflict; a lost
network response cannot be safely assumed to mean no change.

Rollback reads only the fixed-size ready marker and bounded active
pointer and CAS-updates one pointer. A unique random nonce in each
pointer value avoids ABA for cooperating activation clients. This does
not guarantee real-time latency and does not migrate application data.

A single broad create/modify/list token covering the whole application
prefix does **not** make historical assets immutable to adversaries
holding that token. Strong immutability requires separate create-only
release authority and narrower mutable-pointer authority. The current
single-token GitHub Action enforces create-only operation by client
convention, not by grant separation.

A fake ready marker or manifest may be supplied by another authorized
writer. Explicit app verify re-reads each asset and checks the declared
SHA-256 digest, but a malicious writer able to replace manifest and
assets together is not authenticated by these hashes. The stable
browser entrypoint reads the active pointer and follows a relative URL;
it is not an independently verifying cryptographic loader.

Applications with absolute asset paths, custom CSP or service-worker
scopes may not function from the versioned URL until their build
configuration is changed. The tool refuses nonregular build files,
unsafe paths, and overlong keys. It leaves abandoned partial releases
in place rather than deleting unknown data.

## GitHub build and provenance

Source builds execute make build locally on the developer machine
or GitHub runner. A build script is arbitrary code: audit the source
commit and dependencies. A disposable OCI builder can reduce privileges,
but no container automatically makes arbitrary source safe. When
provided, its image is run on the same CI runner, never on a hosting
machine and never with the deployment token.

The Action authenticates uploads only after local build completion.
It verifies the returned app name and Git commit before writing
workflow outputs. A lost HTTP activation reply requires an
authoritative app current inspection before retrying. The tool does
not assume a separate deployment server, SSH, filesystem symlink or
static web server. The existing Skrynia HTTP server serves all bytes.

## Administrative grant recovery

Optional grant provisioning writes a private journal before remote creation, saves the one-time issuer response before user issuance, and marks `issuing_user` on disk before sending that request. `admin inspect` reports recovery state without printing capabilities. A failed/uncertain user-token request may have minted an unrecorded token. For that state, `admin resume` refuses to reissue unless the operator explicitly passes `--accept-duplicate-token-risk true`. Reissuance cannot revoke an orphaned token or guarantee exactly-once semantics. A lost grant-creation issuer response is unrecoverable through this client; repeating `POST /grants` would create an additional grant, so the CLI refuses automatic recovery. Private journal changes serialize under a persistent owner-only filesystem lock.


## Explicit revocation and uncertain DELETE

`admin revoke-user` and `admin revoke-grant` use the locally held issuer capability in the private child-grant journal. The CLI requires the exact grant ID again before either destructive **authority** change. It persists an in-progress state before the HTTP DELETE and requires explicit acknowledgement to retry an uncertain result. A 404 is treated as confirmed absence only with the expected Skrynia `user_not_found` or `grant_not_found` error code; an arbitrary 404 does not confirm revocation. Once done, corresponding credentials are removed from the journal; `admin inspect` never reveals them. Root credentials are not a supported journal input. A revoked capability can still exist as a copied secret elsewhere but must be rejected by Skrynia if revocation was committed.

Grant revocation does **not** erase stored bytes; do not confuse authority lifecycle with data retention or application release lifecycle. Deleting a child grant also affects subordinate users/grants, so treat `revoke-grant` as a privileged operation.

## Browser URL and credential boundaries

The browser client rejects non-HTTP(S) base URLs, embedded usernames/passwords, fragments and queries. This matches the native client's URL policy and avoids treating credentials embedded in a URL as a supported authentication workflow. Browser tokens are supplied explicitly as optional capability strings and sent only on authenticated operations; public GETs remain unauthenticated. Browser JavaScript cannot hide tokens bundled in public code, and browser cross-origin requests still require appropriate server CORS headers.

## Exact HTTP commitment and read-response statuses

The native and browser clients do not infer that *every* 2xx status means an operation completed. Skrynia's versioned contract returns **201 Created** for object creation and **200 OK** for confirmed replacement/deletion and grant operations; object GET and list results must also be **200 OK**. `409` is the create collision, `412` the CAS mismatch, and `404` the missing object. Unexpected successful responses (e.g. `202 Accepted`, `204 No Content`, or `206 Partial Content`) are reported as contract violations rather than silently treated as committed writes or complete reads. A mutation receiving an unexpected success status may still have taken effect, so the caller must reconcile it with an authoritative read before retrying.

## Bounded API metadata and diagnostics

Native client error responses are read with a 4 KiB cap, rather than materializing an arbitrarily large error body. Expected JSON responses (pagination metadata and grant issuance/revocation) have a 1 MiB maximum, enforced both through Content-Length and through a streaming read bound for responses with unknown length. Overlarge responses are explicit errors. **Raw GET object payloads are intentionally not subject to the JSON cap**, since applications can store arbitrary byte objects; applications needing stricter object-size budgets should implement those at their own data protocol boundary. Browser fetch still follows its own memory model and must not be mistaken for a bounded streaming client.

## Isolated real-server credential lifecycle regression

The source-pinned Skrynia CI compatibility job now also exercises `admin create`, `revoke-user`, `revoke-grant` and repeated completed revocation against a disposable localhost server and grant hierarchy. It confirms a revoked user is rejected when attempting a new object creation, a revoked grant's former issuer cannot issue a user, the journal has shed its secrets, and the bytes published before revocation remain readable. All test credentials are private under a temporary owner-only directory, never reused from a production deployment. This checks server/client interoperability but not physical power-loss recovery or complete credential-lifecycle security.

## Generic grant CLI and root discovery

`admin grant --root-grant` reads the generated root grant from the
private state directory selected by **`SKRYNIA_DATA_DIR`**, the exact
same variable as the Skrynia server. It never assumes an HTTP root
credential endpoint or silently falls back to another path. An unset,
inaccessible or invalid directory causes a clean failure before grant
creation. It issues a transient root user token through the normal
Skrynia HTTP issuer endpoint; it does not accept issuer credentials
as ordinary mutation authority.

The CLI prints only the new grant ID and path to its private credential
file. Files and lock files use 0600 permissions; the default credential
directory is owner-only. Uncertain grant creation leaves a pending
record rather than blindly repeating the mutation. Never publish
issuer credentials into GitHub Actions or application assets.
