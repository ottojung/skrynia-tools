# skrynia-v3-trusted-edge-gateway

Minimal internal trusted edge gateway for Skrynia v3 app origins.

This service is **not** Skrynia kernel code, not Program 0, not a CLI, and not
an administrative/deployment tool. It is a separate transport-only edge service
intended to sit behind the public static/edge host (e.g. nginx) that is
configured fail-closed for `/_skrynia/` per skrynia-apps PR #300.

## Trust boundary and behavior

- Accepts only `POST /_skrynia/` (and `/_skrynia`) with `Content-Type:
  application/octet-stream`.
- Enforces per-app allowlists for `Host` and `Origin` (must resolve to the same
  configured app; wrong host/origin/sibling denied). `Host`/`Origin` are
  self-asserted request headers (non-browser clients can spoof them); security
  depends on possession of the correct per-app edge capability plus this binding.
- Requires separate edge client/session/capability authentication via
  `X-Skrynia-Edge-Capability` (separate client secret, **not** the operator/P0
  token). Capabilities must be unique across configured apps.
- Loads a private operator token from a regular file that must have no
  group/world permissions (0600 on Unix) and contain exactly 32 raw bytes.
  Operator config file must also be 0600 on Unix.
- **Only after positive auth** prefixes the opaque request body with P0's
  required header `0x01 || 32-byte raw operator token || opaque-body` and
  forwards it to the configured loopback kernel upstream. The browser/static
  edge must never know, inject, log, or expose the operator/P0/root token.
- Forwards request/response as bounded streams (no payload interpretation,
  no full-body buffering) and handles disconnect/cancellation; transport only
  (ordinary WASM/guest performs per-object grants/CAS/authorization).
- Uses a reqwest client with `no_proxy()` and redirect policy `none`. Does not
  trust or forward `X-Forwarded-*`/`Forwarded` headers; upstream request carries
  only constructed `Content-Type: application/octet-stream` and streamed body.
  Credentials/tokens/capabilities are not written to logs.

## Configuration

Set `SKRYNIA_EDGE_GATEWAY_CONFIG` to a JSON config file path. The config file
must be a regular file without group/world permissions (0600 on Unix).

- `bind` must be loopback-only.
- `upstream` must be `http://` with a numeric loopback IP host only (no DNS).
- `operator_token_file` must be 0600 (Unix) and contain exactly 32 raw bytes.
- `edge_capabilities` are client-facing edge secrets, separate from operator token.
- `apps` must define per-app `hosts`, `origins`, and `edge_capabilities`; hosts,
  origins, and capabilities must not be shared across different app identities.
- `max_body_bytes` bounds request and response streaming (default 16 MiB).

## Tests

Live localhost integration tests use the canonical kit environment variables:

```sh
SKRYNIA_V3_SERVER_BIN=/path/to/skrynia-server \
SKRYNIA_V3_PROGRAM0_WASM=/path/to/skrynia_program0.wasm \
cargo test --manifest-path v3-trusted-edge-gateway/Cargo.toml --tests -- --nocapture --test-threads=1
```

Coverage includes: positive opaque echo, negative auth/host/origin and
cross-app bearer denial with no secret leakage to stderr, 13MiB full-duplex
streaming, cancellation/recovery, chunked request without `Content-Length`
over configured bound (fail-closed + liveness), and config rejection for
duplicate edge capabilities across apps.

## Coordination

- skrynia-apps PR #300: static edge `/_skrynia/` fail-closed until a separate
  authorized trusted gateway is explicitly configured.
- skrynia-tools web admin PR #44: separately hosted admin/service work (distinct
  trust boundary from this transport gateway).
- Antonina security issue: `uuy7k4jCuQ8pewXerGbBve-aptly`.

No production deployment, DNS/TLS changes, Ivanka, or real credentials are part
of this implementation.
