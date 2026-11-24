# Skrynia Tools (native v3)

Reusable **native** components for [Skrynia](https://github.com/ottojung/Skrynia):
an optional operator-supplied dumb Program 0 token gate, an ordinary nonzero
WASM service, a native Rust/WASM client, CLI-free native web administration,
an outbound WebPush sender and a real-stack acceptance harness.

Administration is web-based. The kernel and root proxy do not implement
application semantics; static hosting is separate.

## Components

- `program0/` — generic constant-time 256-bit token gate and opaque relay/
  callback gate only. The root token is operator-held, never in browser JS.
- `native-service/` — ordinary nonzero WASM service: exact-key authorized
  objects, durable push subscriptions/change hints, fixed-slot wake and bounded
  pending-hint processing, sealed code identity and constant-work revision CAS.
- `src/client`, `src/wire/native`, `src/cas`, `src/log`, `src/event_set`,
  `src/browser` — native binary POST client with exact read/write capability
  separation and strict frames; generic log/CAS/snapshot/EventSet algorithms
  are optional client protocols. EventSet requires an injected enumerating store.
- `web-admin-v3/` — static native admin front-end on a separate trusted
  origin. The browser sends opaque binary POST bytes through an operator
  gateway; root/issuer/P0 tokens stay server-side. Activation uses native
  opcodes and a single revision CAS, never AppPut/AppGet (6/7).
- `webpush-sender/` — independently operated least-privilege WebPush sender.
- `fullchain-integration/` — disposable real-stack acceptance with runtime
  SHA256 pins (development evidence only).
- `v3-trusted-edge-gateway/` — authenticated trusted-edge gateway.

## Qualification

Run the exact gates documented in `AGENTS.md` and the draft PR: default
rustfmt, structural lints, workspace tests, zero-warning Clippy, native admin
deterministic tests, native browser WASM fixture, sender tests and the real
full-chain harness. Board coordination: `w4wgIqJnlASKGYrl1UGd8H-realise`,
`NHARAlNNTdqYZJHJaqdD8I-splash`, `ZZYan9Ny0XiLvSBD1iIwCF-hall`,
`5zNqlV9UeXpMI9D84rIEBo-countries`. No production cutover.

Free Software under GNU AGPLv3 or later.
