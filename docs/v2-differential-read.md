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
