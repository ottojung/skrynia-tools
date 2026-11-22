# DRAFT — disposable WebPush / native-P1 loopback integration harness (marceline-dev)

**Do not merge. Draft for Antonina board handoff only.**
Branch: `antonina/webpush-loopback-integration-marceline-dev-20261122`
Base: `470000f2ccedd83eb9dad00f256b59f07a6ba127` (durable replay ledger draft)
Repository: ottojung/skrynia-tools

## Scope
Adds a new, isolated, disposable harness crate `loopback-integration/` plus
`BLOCKER.md`, `README.md`, a reproducible audit script and a DRAFT PR/board
handoff. No changes to production, `main`, live secrets, DNS, certificates,
real subscriptions, benchmark data, Ivanka/MiCall/PX/Viche, or kernel/P0/P1
business logic. PR76/PR322/PR323 source branches untouched. Reuses the warm
shared Cargo target (no heavy duplicate build; ~2-3 s incremental).

## Gating result (explicit blocker)
The pinned P1 WASM does **not** contain push opcodes 16/17/20-23. Reproducible:
`python3 loopback-integration/audit/pinned_p1_push_opcode_audit.py` exits 2 with
exact SHA256 for all seven candidate pinned P1 artifacts. Push-enabled source
exists (`a4717fc`), but no compiled pinned P1 carries the code. Therefore a
real-binary full-stack E2E is **not** claimed.

## What the narrow test proves
Real sender transport (pinned-IP HTTPS adapter with disposable CA), real ES256
VAPID signing, real RFC 8291 encryption, real durable ledger, driven by a loopback
P1 fake and a controlled local HTTPS receiver. Seven deterministic scenarios
with exact, reproducible SHA256 evidence:

- delivery + scoped bearer + subscribe/revoke + durable hint/ACK
- restart suppression (no duplicate send)
- revoke stops delivery
- response rejection (401 => no ACK)
- bounded sender retry (503 + Retry-After => 3 attempts, 2 backoffs)
- forged scoped bearer rejected (P1 code 5)
- no VAPID private key in browser-facing sources; disposable test key

## Explicitly untested boundary
Real pinned P1 WASM push execution, kernel + dumb Program0 + trusted-edge chain,
and browser Service Worker notification. Full-stack acceptance is **not** claimed.

## Checks
`cargo fmt --all -- --check` OK; `cargo test --offline --locked --all-targets
--features test-transport` 69 tests OK; source_lints 7/7 OK; strict clippy OK;
harness `HARNESS_OK 7`, evidence byte-identical across repeated runs.

Volodyslav convention: a pinned push-capable P1 would be the missing key to this
particular lock; until then the loop holds only at the sender boundary. @ottojung
