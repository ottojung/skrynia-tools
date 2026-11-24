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

Private `open_disk` storage is not a static website root. Only the root program
imports `syscall.publish`/`syscall.unpublish`; evaluated children cannot import
that namespace. A dedicated generic publication root program is used for the
publication lifecycle, while Program 0 remains a dumb token-checking proxy with
no app policy and no mailbox authority. The root ABI writes by default to the
private user root, so `publication-requests/<8 lower hex>` lives on that disk;
the provider mounts only that directory read-only and nginx never mounts it.

The root authenticates the operator token, accepts a bounded UTF-8 relative
path, rejects absolute/traversal/NUL/newline/backslash paths, empty or `.`/`..`
segments, oversize input and private roots (`security`, `programs`,
`publication-requests`, `operator`, `private`, `tokens`). Tickets come from
secure randomness with create-only open and bounded retries, so concurrent
requests cannot overwrite an existing ticket file while it exists. A random u32
is collision-protected only by the currently existing file, not by a global
never-reuse guarantee. The request file is written, synced and closed before the
import; it is removed after the import returns on every non-trapping path.

Outcome is reported in the response: 0 means the provider confirmed success;
1..6 are provider failures; -1 invalid request/path, -2 mailbox failure before
the import, -3 unknown outcome (the import may have taken effect), -4 cleanup
failure after a confirmed result. A trap produces no response and is therefore
an unknown outcome, never a success. A pre-invoke write or close failure removes
only the ticket file the root just created, never a pre-existing colliding file;
a trap or uncertain outcome can still leave a ticket. No age-based janitor or
automatic reclamation is implemented. An operator may reclaim such a file only
in a quiesced maintenance window after the provider and all potentially
in-flight requests for that exact ticket are known finished, keyed on the exact
ticket name. Never reclaim by age alone while requests could still be in flight;
the provider never writes or deletes requests.
A deployment helper that POSTs directly to the provider is a convenience path,
not a WASM lifecycle; only the compiled root program exercises the syscall.

Revoke a public entry before deleting or replacing the private bytes it names.
Otherwise a surviving symlink can resolve to later file bytes. Version switches
and rollback revoke the old entry before publishing the selected immutable
target.

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

Publication requires explicit authorization before the provider receives a
private ticket. The public nginx tree contains only explicit publication
links; the source disk is never directly served. Only the provider writes
the publication tree; published source ancestors remain controlled.

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
