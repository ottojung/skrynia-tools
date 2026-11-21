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
  configured app; wrong host/origin/sibling denied).
- Requires separate edge client/session/capability authentication via
  `X-Skrynia-Edge-Capability` (separate client secret, **not** the operator/P0
  token).
- Loads a private operator token from a regular file that must have no
  group/world permissions (0600 on Unix) and contain exactly 32 raw bytes.
- **Only after positive auth** prefixes the opaque request body with P0's
  required header `0x01 || 32-byte raw operator token || opaque-body` and
  forwards it to the configured loopback kernel upstream. The browser/static
  edge must never know, inject, log, or expose the operator/P0/root token.
- Forwards request/response as bounded streams (no payload interpretation) and
  handles disconnect/cancellation; transport only (ordinary WASM/guest performs
  per-object grants/CAS/authorization).
- Uses a reqwest client with `no_proxy()` and redirect policy `none` to avoid
  credential leaks. Credentials/tokens/capabilities are not written to logs.

## Configuration

Set `SKRYNIA_EDGE_GATEWAY_CONFIG` to a JSON config file path. The config file
must be a regular file without group/world permissions (0600 on Unix) because it
may contain edge capability secrets.

```json
{
  "bind": "127.0.0.1:18080",
  "upstream": "http://127.0.0.1:17380/",
  "operator_token_file": "/etc/skrynia/operator.token",
  "max_body_bytes": 16777216,
  "apps": [
    {
      "name": "along",
      "hosts": ["long.a.vau.place"],
      "origins": ["https://long.a.vau.place"],
      "edge_capabilities": ["<separate-edge-capability-secret>"]
    },
    {
      "name": "kawunapp",
      "hosts": ["kawun.a.vau.place"],
      "origins": ["https://kawun.a.vau.place"],
      "edge_capabilities": ["<separate-edge-capability-secret>"]
    },
    {
      "name": "tesicca",
      "hosts": ["tesicca.a.vau.place"],
      "origins": ["https://tesicca.a.vau.place"],
      "edge_capabilities": ["<separate-edge-capability-secret>"]
    },
    {
      "name": "theorem-lab",
      "hosts": ["theorem-lab.a.vau.place"],
      "origins": ["https://theorem-lab.a.vau.place"],
      "edge_capabilities": ["<separate-edge-capability-secret>"]
    },
    {
      "name": "sbp",
      "hosts": ["sbp.a.vau.place"],
      "origins": ["https://sbp.a.vau.place"],
      "edge_capabilities": ["<separate-edge-capability-secret>"]
    }
  ]
}
```

- `bind` and `upstream` must be loopback-only.
- `operator_token_file` must be 0600 (Unix) and contain exactly 32 raw bytes.
- `edge_capabilities` are client-facing edge secrets and must be kept separate
  from the operator token.
- `max_body_bytes` bounds request and response streaming (default 16 MiB).

## Tests

Unit-style checks compile with the crate. Live localhost real kernel+P0+guest
integration tests are in `tests/live.rs` and use the canonical kit environment
variables:

```sh
SKRYNIA_V3_SERVER_BIN=/path/to/skrynia-server \
SKRYNIA_V3_PROGRAM0_WASM=/path/to/skrynia_program0.wasm \
cargo test --manifest-path v3-trusted-edge-gateway/Cargo.toml --test live -- --test-threads=1
```

They cover negative auth/host/cross-origin, positive opaque echo, no secret
leakage to stderr, 13MiB full-duplex streaming, and client cancellation.

## Coordination

- skrynia-apps PR #300 (`@95cc072`): static edge `/_skrynia/` fail-closed until a
  separate authorized trusted gateway is explicitly configured.
- skrynia-apps Along #298: client/native migration coordination context.
- skrynia-tools web admin PR #44: separately hosted admin/service work (distinct
  trust boundary from this transport gateway).
- Antonina security issue: `uuy7k4jCuQ8pewXerGbBve-aptly`.

No production deployment, DNS/TLS changes, Ivanka, or real credentials are part
of this implementation.
