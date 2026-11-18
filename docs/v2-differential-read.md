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
