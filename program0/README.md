# Skrynia v3 — dumb proxy root (Program 0)

This optional WASM is a pure one-to-one byte proxy to one operator-installed
unprivileged guest at programs/root.wasm within Skrynia's confined data
directory. It does not decode requests, route application names, maintain a
registry, authorize users, modify persistent application data, manage
versions, store grant secrets, or implement an administration interface.

It receives two forward-only root file descriptors and consumes only the
33-byte authentication header. After verification, it loads the fixed
operator-supplied guest WASM and passes the original input descriptor (now
advanced past the header) and output descriptor directly into generic
skrynia.eval(code, code_len, input_fd, output_fd, callback). The child
reads and writes through bounded streaming channels, with no copying or
buffering of request/response bodies in Program 0. The whole guest binary
is loaded into memory up to the 8 MiB module limit; that is independent of
request size.

This version is for generic two-FD Skrynia eval (kernel branch
feat/generic-fd-eval-abi), NOT the old 7-argument buffered eval API.
The child is unprivileged: it cannot import root filesystem, global RAM
or network APIs. With callback=-1 the child cannot call skrynia.invoke.
Consequently this is a verified authenticated streaming transport, but
not yet a working application storage/API delegation mechanism; a
reviewed generic callback proxy is still required. Missing, invalid,
or absent child modules fail closed; there is no legacy fallback.

The name programs/root.wasm denotes an operator-selected ordinary child,
not an independently recognized kernel Program 1 identity. The kernel
still only knows the root WASM supplied at SKRYNIA_ROOT_WASM. The kernel
does not know anything about the token or the child module.

NO automatic migration from the former root-owned registry is performed.
That previous code is retained separately as unshippable implementation
material, not as Program 0. Durable application capability services,
code version history, authenticated rollbacks and multi-tenant dispatch must
be reimplemented in ordinary user programs with a separately reviewed
capability boundary. Do not deploy to production from this intermediate state.

## Required operator token

The provided Program 0 rejects every request unless it starts with this
fixed binary authentication header:

    0x01 | 32-byte raw bearer token | opaque request bytes

The version and 32 token bytes are **consumed by Program 0**. They are
never included in the input passed to the evaluated user program. All
request bytes after the header remain opaque and unmodified. Short or
unsupported headers, wrong tokens, missing/invalid verifier files, and
I/O errors fail closed with no application response.

The operator provisions a **cryptographically random 256-bit token** out
of band to exactly the trusted clients that need access. Never place that
token in a repository, public web bundle, URL, access log, or deployment
manifest. In a browser, require explicit user entry and retain it only
in transient session memory, never in static JavaScript. Always use HTTPS
end to end; otherwise possession of the request body allows replay.

Instead of storing the bearer token itself, Program 0 reads the raw
32-byte SHA-256 digest of that token from this private, operator-owned
file under the configured Skrynia data root:

    security/program0.sha256

This is a **binary digest**, not a text hex string. Provision the parent
directory with mode 0700 and the file with 0600, before starting the
server. Generate the token with a cryptographically secure generator,
compute SHA-256 of its **32 raw bytes**, store only that digest on disk,
and deliver the token to authorized clients via a secure out-of-band
channel. Do not expose the verifier through any app or static host.
The operator can rotate access by atomically replacing this digest;
Program 0 reads it afresh for every request, so previously authorized
tokens stop working immediately. Deleting or corrupting the verifier
disables forwarding.

The token is a **root-level bearer capability**: anybody possessing it
can make any API request that this Program 0 chooses to proxy. It is
not a substitute for per-application, per-object or delegated grants.
Authentication is mandatory even when the forwarded request is empty.
Skrynia itself has no knowledge of this header, token, or verification.
The current kernel returns an HTTP error on auth denial; no distinct
HTTP 401 response is implemented at the transport layer.

The authentication header and the body are independently handled:
Program 0 checks and removes only the header and delegates the two
remaining streams to the generic eval API. The current fixed demo guest
path is not a production application registration policy.
