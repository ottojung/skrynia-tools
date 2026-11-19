# Pinned v2 public-read compatibility check

The native Skrynia v2 comparison baseline is
`ottojung/Skrynia@10cbf0d5ad00224e1375a4f2482965f8d3513559`.
Compile its `skrynia-server` binary independently. The optional test
`tests/v2_read_parity/` launches it with an isolated temporary data directory
and a known test object; no existing server or data tree is accessed.

```sh
SKRYNIA_V2_SERVER_BIN=/absolute/path/to/pinned-v2/skrynia-server \
cargo test --locked -p skrynia-tools --test v2_read_parity -- --ignored --nocapture
```

The test compares the v2 server's actual HTTP response with the
`skrynia-tools` v2-compatible facade: GET, HEAD, missing objects,
explicit and suffix ranges, invalid/multi-ranges, 206/416, validators
including weak `If-None-Match`, `If-Range`, plus content-type, length,
ETag and cache headers. The facade explicitly models the empty HTTP
response body length on 416 and JSON error content length.

This is **not a complete compatibility proof**. It does not test the v2
grant tree, tokens, delegated authority, mutation CAS/quota accounting,
subscription delivery, Web Push, concurrent writes or migration. Those
operations are still release blockers; no production cutover is authorized
on the strength of this read-only suite.

The pinned baseline suite also exercises *negative* object-mutation requests
(POST/PUT/DELETE without a valid bearer credential, malformed keys, and
forbidden/malformed conditional headers). Those are verified against actual
v2 HTTP behavior. Positive mutation handling is wired through an injected
`Store::mutate_object` capability, which **must atomically** enforce
grant authority, SHA-256 preconditions and quotas. A real v3-backed
implementation of that capability is not yet present; the default returns
`not_implemented`. Do not treat the passing negative-path differential test
as proof of positive mutation compatibility.

## Disposable root-grant mutation differential

The same opt-in differential command also creates a **new root user token in
the disposable v2 server** by reading only that test server's root grant
record and calling the v2 token-issuance endpoint. It never reads real
production credentials. It then compares positive POST/PUT/DELETE mutations
and read-after-write against a synchronized, root-unlimited in-memory
compatibility model, including conflict, correct and incorrect SHA-256 CAS,
and missing-object deletion. Bodies, status codes and selected HTTP headers
must match the pinned native server.

The in-memory backend is a test oracle, **not** the implemented v3 persistent
grant/quota engine. Child delegation, quotas, revocation, simultaneous writes,
Web Push, and durable migration are still unqualified release gates.

The compatibility library also implements the exact v2 location matcher
grammar (wildcard, exact key, prefix ending in `*`, directory subtree ending
in `/*`, and the empty matcher), strict subset logic and the 32/64-digit
lowercase hexadecimal credential grammar. These are pure helpers reused by
future v3-backed grant authority code, validated with unit cases against the
pinned native v2 definitions. They do **not** yet constitute a complete grant
tree, quota or revocation implementation.

## Reusable grant-core implementation

The library now contains `wire::v2compat::grants::Authority` for
grant/user-token authority. It parses v2 JSON grant/user record structures,
hashes presented credentials, differentiates issuer from user authority,
checks operation and location matchers, and rejects duplicate IDs/tokens,
orphaned grants, cycles, widened child scopes, and widened operations before
accepting the graph. The child-spec validator exactly follows v2's field,
operation, delegation and safe-integer limits. Its quota calculator accounts
for matching object payloads plus 512 bytes, 2048 per descendant grant,
128 for each additional user credential, and 4096 bytes/one object per
sponsored push subscription. Capacity checks apply to the acting grant and
its ancestors; unrelated overlapping grants do not veto mutations.

The disposable positive-mutation parity model uses this authority graph.
**It is not yet a durable production engine**: it does not persist grant
creation, revocations, mutations, concurrent transactions, or subscriptions
through the root WASM. Those are release-blocking implementations, not
satisfied by the validated pure rules and synthetic oracle.

## Restricted-grant quota differential

The isolated v2 differential fixture also creates a delegated child grant
whose location is `team/*`, grants create/modify/delete, and limits quota to
3,000 bytes and one object. It compares the actual v2 HTTP results against the
pure authority/quota-backed compatibility model for successful creation,
quota-boundary rejection, replacement/shrinking, object-count exhaustion,
deletion and reuse, and a write outside the grant scope. Negative SHA-256
precondition failures take precedence over quota failures exactly as v2 does.

The fixture identifies the unique root grant by the explicit `parent: null`
field rather than relying on directory iteration order. All data and newly
issued credentials exist only in the short-lived test directory. These tests
validate **selected** cases, not durable v3-backed quota enforcement or
complete concurrency/metadata/subscription parity.

## Injected durable-backend prototype and uncertainty

The DurableBackend prototype stores grant/user records and object contents
behind injected StateStore and RandomSource capabilities. State changes are
calculated on a private clone, validated, and submitted to StateStore::save
while the authority lock is held. Only a confirmed save updates the
in-process committed snapshot. Since a failed save might still have
committed externally, any save error permanently latches that backend
instance into state_uncertain (503) for BOTH reads and writes until
restarted and reloaded. It never silently retries an ambiguous mutation.
Tests cover failure before persistence and lost acknowledgment after save.

The grant graph validator rejects duplicate user/issuer credentials and
collisions between user and issuer roles; token issuance cannot persist
such a collision. Loaded nonempty grant graphs are validated before use.

IMPORTANT: this is not a production v3-backed implementation. StateStore
is injected, not yet a real Program 0 kernel-file adapter. The prototype
serializes the entire authority/object map for each mutation, so memory
and work scale with total stored data. The tests do not establish a full
v2 HTTP facade, push, concurrent cross-process CAS, filesystem-level
recovery after ambiguous writes or production migration.

## Subscription delivery queue failure semantics

The experimental v2 Web Push compatibility queue has an injectable
Transport and Durable capability. Durable::load and Durable::save report
errors; a failed initial load prevents opening the queue. Enqueue/removal,
acknowledgment, Gone, and retry transitions persist a candidate snapshot
BEFORE publishing it in the local queue. An ambiguous save error latches
the queue as state_uncertain, refusing both further deliveries and reads
until a new instance reloads the underlying store. A lost acknowledgment
can still produce a duplicate delivery after restart (at-least-once
semantics); it must never cause immediate retries against an uncertain
state. Deterministic failure injection tests cover failures before saving,
after saving but before acknowledgment, and initial load failure.

This is a protocol component, NOT a complete Web Push implementation:
the transport, encryption, durable filesystem adapter, subscription
quota integration and externally exact v2 response semantics remain
separate verification gates. The queue reserializes its full pending list
on each state change and is not yet suitable for a high-volume service.

## Combined backend and subscription integration boundary

The combined draft wires the v2 HTTP routing traits together and exercises
injected authority, mutation and subscription components in one Rust workspace.
Until the subscription store and outbound transport are actually bound to
the kernel-backed authority, DurableBackend deliberately returns a visible
501 not_implemented for push routes. It must never claim compatibility by
falling back to a test-memory subscription implementation. Both injected
state machines retain their fail-closed persistence uncertainty behavior.

## v2 grant HTTP parity (injected backend draft)

The v2 compatibility facade now supports POST /grants, GET /grants/:id,
POST /grants/:id/users, DELETE /grants/:id/users/:userId and
DELETE /grants/:id via its injected GrantAdmin trait. Issuer-token
management and user-token scope/inspection are distinct; malformed
or absent authorization is rejected without leaking sensitive values.
Creation enforces the v2 JSON size/spec validation, returns a random
32-hex grant ID and a 64-hex issuer token, and inspection redacts
issuer/user secret hashes. Unrecognized grant methods and paths return
404, matching the pinned v2 HTTP router.

An opt-in disposable-server differential suite compares live pinned v2
responses with the injected durable-backend facade for delegated grants,
user issuance, inspection, invalid specs and missing authorization.
Random identifiers and bearer secrets are mapped between the two test
instances, not compared as equal random bytes. THIS TEST IS NOT A
production v3 persistence or migration qualification.

## Separate reusable v2 userland components (composed draft)

The optional injected v2-compatible grant authority, browser-facing object
listing/immutable-release CAS protocol, and durable push subscription
records can coexist in one tools workspace with all existing unit tests.
Store listing uses the persisted grant auth and returns only object keys,
not subscription records. Subscription metadata is counted in grant quota
usage but is NOT considered an application object in the list API.
This composition preserves one-time issuer/user credential distinction,
private token handling and the fail-closed uncertain-save latch.

This is **not** an end-to-end v3 HTTP compatibility proof. The native VM
still exposes only POST /, the injected facade still requires an external
transport adapter, the actual Program 0 kernel-backed authority path is
not yet the complete persisted grant tree, and push delivery/acks/VAPID
remain separate gates. Browser code activation/rollback concerns code
releases only, never data migrations. Keep production cutover blocked.
