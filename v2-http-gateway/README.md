# Optional trusted HTTP adapter for the ordinary v2 program

This is a separately operated HTTP service, **not** Skrynia kernel code,
not Program 0, not an administrative CLI, and not a substitute for the
ordinary v2 WASM provider. The kernel accepts only binary POST / and
knows only the independently supplied Program 0, never Program 1.

The adapter translates legacy v2 HTTP method/path/query/headers/body
into a JSON v2 Request and sends the following body to the v3 kernel:

    0x01 || 32 raw operator-token bytes || v2 request JSON

Program 0 strips/authenticates the header and executes a user-supplied
v2 guest; its response is mapped back to HTTP status, headers and body.
Neither the adapter nor Program 0 implements v2 grants, quotas or
application authorization. Those belong in the ordinary v2 program.

## Operation and trust boundary

Only environment configuration is supported (no administrative CLI):

- SKRYNIA_V2_GATEWAY_BIND: loopback-only socket, e.g.
  127.0.0.1:19081. Public/wildcard binds are rejected.
- SKRYNIA_V3_URL: exact loopback-only upstream URL with explicit port
  and path /, e.g. http://127.0.0.1:19080/. Remote hosts,
  alternate paths, credentials, fragments, queries and TLS endpoints
  are rejected.
- SKRYNIA_OPERATOR_TOKEN_FILE: operator-controlled **raw 32-byte**
  bearer token file, not the SHA-256 verifier used by Program 0.
  It must be a regular file (not a symlink) without any group/world
  permissions. Use mode 0600, under private operator-owned parent
  directories. The token file is reloaded on every request for
  immediate credential rotation/revocation.

Do not place the operator token in JS/HTML bundles, public URLs,
Skrynia objects, logs or repositories. The adapter and v3 upstream
must be on the same trusted host/network namespace. The outbound
HTTP client disables environment-inherited proxies and redirects:
the bearer is never forwarded to a remote URL via either.

For remote legacy clients, configure a **separate trusted HTTPS
reverse proxy**, with appropriate access restrictions. The adapter
must remain loopback-only, not publicly exposed directly. A
reverse proxy exposing it publicly is security-sensitive because
the adapter does not authenticate legacy users. When positive
mutations are implemented, the v2 guest must enforce the original
v2 user/issuer credential and grant rules. Anyone possessing the
root Program 0 bearer can access broadly privileged v3 operations.

## Limits and evidence

This is an adapter only for the **currently implemented read-only
subset** of the ordinary v2 guest. Inputs are buffered as JSON
(up to 8 MiB), responses up to 10 MiB, upstream timeout 30 seconds.
Upstream failures report 502, missing/corrupt/insecure token 503
and oversized requests 413. Mutations are never automatically
replayed after an uncertain upstream failure.

Real four-layer integration tests exercise legacy HTTP -> gateway ->
compiled Rust Program 0 on generic v3 kernel -> independently built
ordinary v2 guest -> actual disposable disk storage. They check
GET, HEAD, byte ranges, preflight, rejected writes, root token
rotation, corrupt and missing verifier, and refusal of insecure
token files. Unit tests reject symlink tokens, wrong lengths, and
group-readable files. A redirect test shows the raw bearer is
not sent to an untrusted redirect destination.

The adapter is NOT full v2 compatibility and not production-ready.
Positive grant-aware mutations, issuer/user credentials,
conditional CAS/quota accounting, Web Push, larger objects,
restart/crash recovery, external TLS and migration of existing
user data remain unfinished. No production deployment or real
operator credential was used in testing.
