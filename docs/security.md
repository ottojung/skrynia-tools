# Security and failure boundaries

**Status:** the native HTTP library, web administration, GitHub publishing
Actions, CAS cell, append log and static releases are implemented and
smoke-tested. The additional fault-injection, adversarial verification and WebAssembly guarantees described here are requirements for future production qualification, not claims about tests already completed.

Skrynia enforces one-key atomic operations and scoped bearer grants. It does not enforce client protocols, guarantee execution by other clients, or provide private reads. Library users must distinguish storage-enforced authority, cooperative assumptions, and independently checked payload validity.

## Credential model

- **Issuer tokens** issue/revoke exact-grant user tokens and revoke the grant subtree; **user tokens** perform operations, including `delegate` when granted.
- Never put **plaintext** root or issuer credentials into public bytes, JavaScript bundles, URLs, logs or Git. The web admin stores grant capabilities only as authenticated-encrypted ciphertext in publicly readable Skrynia objects, using a high-entropy owner-held access secret. Anyone with the ciphertext and the access secret can decrypt them; this is not server-side private storage.
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

## Retired v2 deployment boundary

The v2 HTTP deployment/bootstrap/action path was removed. Native releases
are sealed and activated through ordinary P1 opcodes behind a trusted
gateway with single revision CAS. No CLI, no `/store` facade, no
browser-held root/P0 token.

A fake ready marker or manifest may be supplied by another authorized
writer. Explicit app verify re-reads each asset and checks the declared
SHA-256 digest, but a malicious writer able to replace manifest and
assets together is not authenticated by these hashes. The stable
browser entrypoint reads the active pointer and follows a relative URL;
it is not an independently verifying cryptographic loader.

Each application should have a stable browser origin across releases.
Redirecting application navigation to the shared Skrynia API origin
mixes untrusted applications in one browser security boundary.
Application page and immutable release paths must be read-only.
An independently scoped `/_skrynia/` API proxy on an app hostname
may forward credential-authorized mutations, but the gateway must not
create privileges or expose a blanket writer token. The Rust Skrynia
server enforces operations and grant scope for every mutation.

Applications with absolute asset paths, custom CSP or service-worker
scopes may not function from the versioned URL until their build
configuration is changed. In particular, registering a worker at
`./sw.js` in a versioned directory scopes it to that version;
the app must independently publish and register a stable `/sw.js`
if it needs persistent worker scope across deployments. The code
publisher does not modify that stable worker as part of rollback. The tool refuses nonregular build files,
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
It constructs the release manifest from the requested app name and exact
Git commit and writes workflow outputs only after confirmed activation. A lost HTTP activation reply requires an
authoritative app current inspection before retrying. The tool does
not assume a separate deployment server, SSH, filesystem symlink or
static web server. The existing Skrynia HTTP server serves all bytes.

## Web-based grant recovery and revocation

The browser admin encrypts parent, issuer and user capabilities in one
publicly readable vault. A high-entropy access secret is kept off the
server and provided at unlock. The vault is not a private Skrynia object;
any party with the access secret can decrypt its contents.

Before making a grant, issuance or revocation mutation, the browser
persists an encrypted `pending` record with SHA-256 CAS. A lost response
may indicate a committed mutation; the browser must not automatically
retry, and a lost one-time issuer response may be unrecoverable. A
failed post-issuance vault write shows an emergency recovery export.
An uncertain vault save blocks further changes until the user reloads
and reconciles the authoritative object. This protects against
blind duplicate requests but is not multi-object atomicity.

The UI requires explicit confirmation for revocations. A confirmed
grant/user deletion removes the matching capabilities from the vault.
Revocation affects authority, **not** retained object bytes, code
releases, or application data. The ordinary Skrynia server enforces
the hierarchy and permissions.

## Browser URL and credential boundaries

The browser client rejects non-HTTP(S) base URLs, embedded usernames/passwords, fragments and queries. This avoids treating credentials embedded in a URL as a supported authentication workflow. Browser tokens are supplied explicitly as optional capability strings and sent only on authenticated operations; public GETs remain unauthenticated. Browser JavaScript cannot hide tokens bundled in public code, and browser cross-origin requests still require appropriate server CORS headers.

## Exact HTTP commitment and read-response statuses

The native and browser clients do not infer that *every* 2xx status means an operation completed. Skrynia's versioned contract returns **201 Created** for object creation and **200 OK** for confirmed replacement/deletion and grant operations; object GET and list results must also be **200 OK**. `409` is the create collision, `412` the CAS mismatch, and `404` the missing object. Unexpected successful responses (e.g. `202 Accepted`, `204 No Content`, or `206 Partial Content`) are reported as contract violations rather than silently treated as committed writes or complete reads. A mutation receiving an unexpected success status may still have taken effect, so the caller must reconcile it with an authoritative read before retrying.

## Bounded API metadata and diagnostics

Native client error responses are read with a 4 KiB cap, rather than materializing an arbitrarily large error body. Expected JSON responses (pagination metadata and grant issuance/revocation) have a 1 MiB maximum, enforced both through Content-Length and through a streaming read bound for responses with unknown length. Overlarge responses are explicit errors. **Raw GET object payloads are intentionally not subject to the JSON cap**, since applications can store arbitrary byte objects; applications needing stricter object-size budgets should implement those at their own data protocol boundary. Browser fetch still follows its own memory model and must not be mistaken for a bounded streaming client.

## Integrated web admin and publishing verification

The disposable Rust Skrynia server and Chromium tests cover web
administration, encrypted vault unlock/upgrade, user and grant
issuance/revocation, all-app release inventory and backward/forward
CAS activation. Failure injection exercises uncertain vault writes
and lost one-time grant responses. Separate tests exercise the GitHub
Action's JavaScript publisher and stable PWA assets.

The browser vault remains a trusted client application. A malicious
publisher who can replace the admin JavaScript could steal credentials
as they are unlocked, so the admin-code publication grant must be
protected independently of the data grants stored inside its vault.
