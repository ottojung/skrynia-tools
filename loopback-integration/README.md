# Disposable loopback integration harness (DRAFT)

Verifies the draft outbound WebPush + native P1 **subscription contract** with
the real sender transport, VAPID signing, RFC 8291 encryption and durable
replay ledger, against a **controlled local HTTPS receiver** (disposable CA,
`127.0.0.1` only). No production / main / DNS / certificates / real secrets /
real subscriptions / benchmark data / Ivanka / MiCall / PX / Viche touched.

## Run

```sh
CARGO_TARGET_DIR=/workspace/skrynia-v3-integration-kit/target \
  cargo run --offline
```

Reuses the warm shared Cargo target; no heavy duplicate build. Requires
`openssl` (disposable per-instance CA/cert) and the `test-transport` feature
(enabled by this crate's path dependency on `webpush-sender`).

## What is proven (falsifiable, exact SHA evidence)

| scenario | assertion | SHA256 of canonical evidence |
| --- | --- | --- |
| A | 201 delivery, `POST /push/x`, Host/SNI `push.example.test`, `vapid t=...`, P1 opcodes `16,20,21,23,22`, scoped cap/ns on every P1 call, durable ledger file persisted | `cbdcca621ebdc0ab068928bc0e0366e0c49c6fd69869894b4f639d6d2d1a69c9` |
| A2 | restart with same grant/store: hint skipped, receiver still saw exactly 1 request | `b09f63dfaedb9f656fe6332a8d2f31a815d580ced6f1b12001b0dda678555191` |
| A3 | revoke (opcode 17), new hint => no delivery, hint still ACKed | `7c3eace8f8300e64c9d45789a895732d8ebaeba032b49b7b737b4c783c6c4c64` |
| B | receiver 401 => `Transport("status")`, hint not ACKed, response rejected | `8807766bc68bb6887a832be65f79fdd8187bcfaa171c19f3d5f6f155299ac766` |
| C | receiver 503 + Retry-After => `Exhausted`, exactly 3 bounded attempts, 2 backoffs | `311eb93ed0b9325d43851c18168d2a54e265d49cc410b599205a62570e93c35f` |
| D | forged 32-byte bearer rejected by P1 with code 5 (`Err(P1(5))`) | `17811d4c6645d26ffb245c8611dd37845b9014a1797da4f4a9fd2233fa976368` |
| E | VAPID private key absent from browser-facing sources (0 hits); wire key is public point; test key is disposable `[3u8;32]` | `a09fb3e92dec5b6cf9e27ba4ac4ab87b484bd9dfd907a44f8cad16f663391ce6` |

Evidence is reproduced by hashing a canonical `name|fields` string; the
`FINAL` lines are byte-identical across repeated runs.

## Explicitly UNTESTED boundary — no full-stack acceptance claimed

1. **Real pinned P1 WASM** executing push opcodes — BLOCKED, see `BLOCKER.md`.
   Opcodes 16/17/20-23 are exercised against a loopback P1 fake, not the WASM.
2. **Kernel / externally supplied dumb Program0 / native trusted edge** — the
   real kernel and P0 binaries were reused/read, but not chained here because
   the P1 they would drive cannot execute the push opcodes (gate #1).
3. **Browser Service Worker notification** — cannot be proven by an HTTP
   receiver alone; no browser is involved. Not claimed.
4. Remote push services, real VAPID keys, real DNS/TLS, real subscriptions.

The harness therefore demonstrates the sender-side, scoped-bearer,
subscribe/revoke, durable hint/ACK, response-rejection, restart and retry
contract with a local receiver — a narrower, falsifiable integration result.
