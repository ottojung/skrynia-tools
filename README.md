# Skrynia Tools

Browser-based administration, CI application deployment Actions, and
reusable client protocols for [Skrynia](https://github.com/ottojung/Skrynia).

**There is no command-line application.** Administration happens in the
web interface. A one-time `deploy.sh` bootstrap publishes that
interface; CI publishes apps via GitHub Actions without a CLI executable.

## Native web administration

`web-admin-v3/` is the CLI-free native administration front-end. It runs as
static assets on a separate trusted origin and talks to an operator gateway
over opaque binary POST bytes. The browser never receives root/P0/operator
tokens; the gateway performs pinned P0 framing. Activation is a single native
revision CAS (opcodes 9-14), never AppPut/AppGet 6/7.

## Native components

- `program0/` — dumb 256-bit token gate and opaque relay only.
- `native-service/` — ordinary nonzero WASM service: exact-key data, grants,
  push subscriptions/change hints, wake generation and constant-work CAS.
- `webpush-sender/` — least-privilege outbound sender (VAPID/ES256,
  AES128GCM, pinned-IP HTTPS, durable replay ledger).
- `fullchain-integration/` — disposable real-stack acceptance harness with
  runtime SHA256 pins (development evidence only).
- `v3-trusted-edge-gateway/` — authenticated trusted-edge gateway.

The retired v2 HTTP client, `/store` bootstrap/actions, browser SDK and
legacy web-admin were removed. Generic CAS/log/event-set client bindings
are recorded as an exact scoped followup in
[docs/native-protocol-followup.md](docs/native-protocol-followup.md) pending
native transport port (owner-coordinated; reserved root protocol library
paths are not modified in this draft).

[Deployment](docs/deployment.md) ·
[Web administration](docs/web-admin.md) ·
[Architecture](docs/architecture.md) ·
[Security](docs/security.md) ·
[Optional protocols](docs/protocols.md) ·
[Contributor rules](AGENTS.md).

Free Software under GNU AGPLv3 or later.
