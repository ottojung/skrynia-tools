# Compiled ordinary exact-key fallback and sender recovery

Development acceptance only. It neither authorizes production release nor
uses real browser credentials, subscriber data, remote push services or DNS.

## Source-pinned runtime

The Rust test harness requires five separate SHA256-pinned artifacts in the
untracked artifacts directory, verified before any process is started:

- Skrynia generic kernel main commit 1c5639debea8838277e5f22d9136ff263e894071,
  SHA256 ac301e18fcabf2cbf9f246fbaf1b7985e4062b6225da125f873138a2ed77401c
- Operator-supplied dumb token-checking P0,
  SHA256 d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa
- Ordinary P1 from tools PR96 head 11dd55a73875e7816c2050a7caa3f81a6a79f6ae,
  wasm32-unknown-unknown cb-guest Rust 1.93 SHA256
  e9e5e1f3543bfbfabce1788a6a9535bed37ef15187ffb50a5aa714d27775ca41
- Trusted edge 9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f
- Echo P2 dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5

The exact artifact requirements and verification live in
fullchain-integration/src/chain/pins/table.rs.

## Actual sender/provider checks

The fullchain-integration/src/scenarios/ordinary Rust source uses a new
disposable kernel and trusted edge with a subscription to one ordinary exact
data key; no reserved wake-key subscription is installed.

The first 256 real P1 CAS updates produce the historical per-sequence hash
hints. A further CAS produces one high-bit-tagged 64-bit dirty generation
representing an opaque reread invalidation. The Rust WebPush sender reads the
generation, VAPID-signs and encrypts it, and sends to a disposable HTTPS push
provider that verifies the ES256 signature and AES128GCM plaintext. The
high-bit generation is ACKed, making 256 old legacy hash hints visible again.

A second sender cycle consumes and ACKs each of the old hash hints, including
their actual digest bytes. Reopening the exact durable ledger in a fresh sender
then reports zero outstanding hints without a duplicate delivery. This
protects against out-of-order numerical sequences after overflow.

The first integrated test ran on 2026-11-23 in 120.66 seconds and ended
FULLCHAIN_OK. The expanded recovery/restart test completed in 157.04 seconds
and ended FULLCHAIN_OK. Logs are host-local at
/tmp/skrynia-ordinary-real-pump-first.log and
/tmp/skrynia-ordinary-real-pump-replay.log. Binaries and private credentials
are never committed. The test harness source and expected pinned SHA256
values are committed for repeatability.

## Unqualified release work

This proof uses a disposable local HTTPS provider, not the browser
PushManager with a real external push service. Application grant provisioning,
offline catalogue migration and backup, five-app cutover, operator TLS/DNS and
production approval remain separate gates.
