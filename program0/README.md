# Skrynia v3 — dumb proxy root (Program 0)

This optional WASM is a pure one-to-one byte proxy to one operator-installed
unprivileged guest at programs/root.wasm within Skrynia's confined data
directory. It does not decode requests, route application names, maintain a
registry, authorize users, modify persistent application data, manage
versions, store grant secrets, or implement an administration interface.

It receives the two forward-only root file descriptors, reads the complete
request, loads the operator's fixed guest module, invokes that WASM with the
opaque bytes, and forwards the returned bytes to output. It provides no
callback for skrynia.invoke and cannot grant an untrusted guest disk/RAM
access. Missing or invalid guest returns an error, never a fallback service.
Input is capped at 16 MiB, child code at 8 MiB, output at 8 MiB. These are
temporary implementation bounds, NOT true streaming: Skrynia's current
eval guest ABI is memory-buffered. To make the proxy fully streaming, the
kernel must expose a safe guest FD execution ABI, preserving child isolation.

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

This branch adds a tested authentication boundary to the earlier
buffered generic-eval example. A genuine streaming, generic descriptor
evaluation/proxy API is still being developed; do not interpret the
fixed demo guest path as an application registration policy.
