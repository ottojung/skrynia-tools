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

The evaluated v2 guest imports only the generic skrynia.invoke,
skrynia.read and skrynia.write operations; it cannot directly import
kernel disk, global RAM or network APIs. Program 0 forwards the
four generic disk opcodes OPEN_DISK, READ, WRITE and CLOSE through
skrynia.invoke. The v2 program itself chooses the application-owned
v2/objects/ prefix for read requests. This is not a kernel v2 namespace
or special Program 1 execution path.

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

The ordinary v2 guest now exports skrynia_main(input_fd, output_fd),
accepting the kernel's two forward-only file descriptors; it decodes
a bounded JSON v2 request and writes the JSON reply to output, retrying
short writes. It implements the shared v2 semantics for OPTIONS
preflight, GET, HEAD, ETag/ranges, invalid paths and missing objects
using root-forwarded generic open/read/close through skrynia.invoke.

Nine native v2 unit tests pass with a fake generic kernel-file API.
The real integration test under tests/v3_proxy/v2 uses an independently
built *actual Rust Program 0* with its operator-provisioned token and
the corrected generic-FD kernel PR #111. It proves that the v2 guest
reads actual stored bytes through the generic callback, serves
GET/HEAD/ranges and rejects invalid/unauthorized requests, while
kernel HTTP remains POST-only. The previous test-only WAT root and
buffered four-pointer ABI are retired; do not treat that old fixture
as production evidence.

Build the v2 guest with the skrynia-tools guest-core-only feature
(enabled automatically here), so browser wasm-bindgen/JavaScript
imports cannot enter the unprivileged VM. Browser builds of the
ordinary skrynia-tools library remain unchanged. The resulting
standalone WASM must import only skrynia.invoke, read and write.

The current v2 response is a bounded JSON envelope, with maximum
8 MiB request and 10 MiB encoded response; the full legacy v2 API
supports much larger objects. Authorization for v2 positive mutations,
issuer/user credentials, persistent grant tree, quota/CAS accounting,
Web Push, fault/restart recovery, legacy external HTTP gateway,
differential v2 compatibility and data migration are NOT implemented.
All unfinished operations must fail closed. Never expose the Program 0
bearer token in public application JavaScript, repositories or URLs.
No production state is migrated or accessed.
