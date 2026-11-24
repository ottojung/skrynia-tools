# Security and failure boundaries

The kernel supplies generic execution and delegated resource operations. The
ordinary native service owns application authentication and storage rules.
A protocol implemented by a client is not an authority restriction.

## Credentials

- The private root bearer is held by the trusted edge and checked by the
  operator-supplied Program 0. It is never in browser code, URLs or logs.
- The edge capability identifies an authorized app-scoped caller. Host and
  Origin checks are complementary, not independent proof of authority.
- The ordinary service addresses data by a 32-byte read capability; the
  corresponding write capability hashes to it. The read capability does not
  confer write authority. A write capability authorizes the namespace, not
  just one key. An app that needs per-object write isolation must use separate
  capability namespaces or an independently enforced mediated guest.
- Program registration and administration require separate authority.
  A public invocation identifier is not an administrator capability.
- The web admin encrypts its local vault using an owner-held access secret.
  Unlocking makes secrets available to trusted admin JavaScript; protect the
  code and origin accordingly. The vault is not a substitute for service
  authorization.

## Mutation outcomes

A valid success frame confirms the specific operation; an explicit conflict
does not. HTTP transport success alone proves neither. The native clients
require status 200, a complete versioned response frame and, for writes,
the expected content digest. Unexpected status, malformed responses and lost
replies can leave the outcome unknown. Reconcile using authoritative reads,
operation IDs or revision history rather than automatically replaying writes.

Untrusted holders of a namespace write capability can overwrite or delete
objects even if their intended client protocol only creates entries. Thus
append-only and immutable snapshot claims require cooperating writers or an
independent service that enforces those restrictions. Digest verification detects
corruption but cannot restore lost data. There is no general safe collector
for the optional client protocols.

## Publication boundary

Private `open_disk` storage is not a static website root. The root program
alone imports `syscall.publish` and `syscall.unpublish`; each call carries one
u32 ticket allocated with create-only semantics, never a guest pointer or raw
publication authority. The matching `publication-requests/<8 lower hex>` file
holds exact bounded UTF-8 relative path bytes with no trailing newline. The
writer never truncates or overwrites another request, invokes the import only
after the request is synced, and deletes its own ticket on success, provider
failure and pre-invoke failure. The provider treats request files as read-only
and shares no private disk mount with nginx.

Path validation rejects absolute paths, traversal, empty/`.`/`..` segments,
backslashes, NUL/newlines and oversize or non-UTF-8 input, and permits only
public regular-file targets under `apps/`. Security material, program code,
operator data and the mailbox itself are never publishable. Children cannot
import the syscall namespace; their mediated requests go through the root
callback, which enforces the same validation after token authorization.

Revoke a public entry before deleting or replacing the private bytes it names.
Otherwise a surviving symlink can resolve to a later file at the same path.
Version switches and rollback revoke the old entry and publish the selected
immutable target rather than relying on storage aliasing.

## Transport and isolation

Use HTTPS except explicit disposable loopback tests. Do not put capability
values in endpoint URLs, diagnostics, source bundles or repository history.
The SDK rejects credentials, queries and fragments in its endpoint URL, rejects
redirects and bounds native response frames to 16 MiB. Browser reads stream to
that bound. Browser origin policy still applies.

Static application origins are distinct from the admin origin and each other.
The edge must not expose the root bearer or create a shared public writer.
Ordinary guest callback scopes must not permit access to the operator's private
filesystem or amplify delegated authority. Test these boundaries against compiled
guests, not only mocked transports.

## Release and recovery

Build the exact source revision in an appropriately constrained environment.
Immutable code identity and a monotonic active revision protect code rollback
against ordinary A-B-A replay. A code rollback does not roll back data.
Separate static asset verification, backup, import, authorization and recovery
plans are required before production promotion.

Web Push is a coalesced invalidation mechanism. Delivery and acknowledgement may
repeat after failures; readers consult authoritative data. The separate sender
needs bounded outbound networking, TLS validation, scoped subscription access
and a durable delivery ledger. See its own implementation and tests.

Local model tests, browser fixtures and pinned disposable integration tests
are distinct evidence; none alone establishes production readiness.
