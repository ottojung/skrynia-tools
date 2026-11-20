# Optional Skrynia v2 API provider as an ordinary user program

This crate is an example of a non-root WASM program. It reuses the v2
HTTP-semantic compatibility code under skrynia-tools/src/wire/v2compat.
It is not a second Skrynia kernel, is not linked into Program 0 and has no
special kernel designation as Program 1.

The Skrynia server knows and launches only the operator-provided Program 0,
which authenticates its own 256-bit bearer and exposes generic v3 APIs.
That program may use generic skrynia.eval to evaluate any further program.
An arbitrary child may be the v2 API provider built here:

    user v2 request as bytes -> generic v3 POST / -> Program 0
                                       | generic eval
                                       v
                               ordinary v2 WASM
                                  | invoke
                                  v
                            delegated v3 API callback

The evaluated v2 guest imports only skrynia.invoke; it cannot directly
access storage, global RAM or network endpoints. In the example, a trusted
parent callback supplies public object reads using a provisional app-defined
binary contract (opcode 1, u16 little-endian key byte length, UTF-8 key).
The callback responds with [0] for missing, or [1, object bytes...] when
found. This is an example delegated callback, NOT a kernel-level v2
endpoint or a general v3 protocol specification.

## Request and reply

Both are bounded JSON-serialized v2compat::{Request,Response} values.
The request has method, path, query, headers, and binary body (JSON byte
array). The reply contains status, headers and body. A string method=GET
is opaque POST-body data to the kernel. No v2 GET, HEAD, PUT, DELETE or
OPTIONS route is installed in Skrynia. The outer v3 HTTP response is
still from binary POST /; the v2 status lives in the inner reply envelope.

An external operator-controlled edge adapter would be required for
unmodified v2 HTTP clients. It must translate methods, paths, headers,
conditional/range semantics and raw bodies to this envelope, and translate
replies back. This edge must be trusted with the root Program 0 token if
it is the caller, and that token must never be given to anonymous website
visitors. v2 user/issuer credentials remain separate from the v3 root token.

## Verified and deliberately unimplemented behavior

The current v2 guest implements the shared v2 semantics for OPTIONS
preflight, GET, HEAD, ETag/ranges, invalid paths and missing objects using
the injected public-read callback. Bad callback responses and backend
unavailability fail closed as HTTP-semantic 503 in the reply. It does
not grant positive mutations; unimplemented grants, Web Push, listings,
health and version APIs do not fabricate success. It is not full v2
compatibility, not a migration, and no production credentials/data are used.

The native unit suite exercises successful reads, HEAD/ranges, unknown
fields, malformed frames, missing callback and unimplemented methods.
An explicitly opted-in real-kernel test compiles a test-only root that
evaluates the separately built v2 guest using generic skrynia.eval and an
in-memory public-read callback. The test ensures the outer server rejects
HTTP GET even when the inner v2 operation is GET.

Build the standalone guest with the skrynia-tools guest-core-only
feature enabled (the guest crate does this automatically), so browser
wasm-bindgen/JavaScript imports cannot enter the VM guest:

    cargo build --locked -p skrynia-v2-provider --release --target wasm32-unknown-unknown

The resulting guest has only the skrynia.invoke import. Rust browser
clients remain fully supported by ordinary skrynia-tools builds,
including its existing --no-default-features browser CI command.

Current limits: a whole-envelope 8 MiB request limit and 10 MiB encoded
reply limit, a temporary nonstreaming representation. Actual v2 supports
larger objects. Positive v2 writes require a durable grant-aware CAS backend
with correct quota and issuer/user-token authority; delegation, revocation,
push delivery, failure recovery, independent differential parity, external
adapter security and data migration all remain release blockers.
