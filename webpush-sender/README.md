# Standalone least-privilege outbound WebPush sender

Scope: **sender only**. Kernel, Program 0, P1, apps, CI/main, real secrets,
production DNS/TLS and Ivanka are untouched. No v2 routes or legacy push
endpoints are introduced.

## Contract

- Consumes ordinary P1 ops under a distinct, writer-issued `Kind::Sender`
  grant: `HintList=20`, `HintRead=21`, `HintAck=22`, `SubFetch=23`.
- The sender capability is a 32-byte bearer scoped to one exact `(ns, key)`.
  No root/operator/browser token is compiled in or read from a browser.
- VAPID private key stays server-side (injected `SigningKey`). Only the JWT
  and the public verification key leave the process.
- Hint is only a wake-up: after `HintRead` the sender **re-reads** `SubFetch`
  for authoritative endpoint material before delivery (hint is not a commit
  proof, per PR64 staging contract).
- ACK only after successful delivery; replayed seqs re-deliver with the same
  `Topic` so push services dedupe idempotently. P1 ACK is already idempotent.
- Bounded payload (<=4096 B, equal to P1 `MAX_SUB_BYTES`), bounded hints,
  bounded subscriptions, bounded workers, bounded retries/backoff.
- All external effects are injected capabilities (`P1Transport`,
  `PushTransport`, `Resolver`, `Clock`, `Rng`). Tests use fake endpoints and
  disposable data; no real DNS/TLS/network in this crate's test suite.

## SSRF posture

HTTPS-only, port 443 only, no userinfo, host must not be an IP literal,
every resolved address must be global unicast (loopback, RFC1918, link-local
incl. 169.254.169.254, CGNAT 100.64/10, multicast, unspecified, docs ranges,
IPv6 ULA/link-local and v4-mapped v6 are rejected). The chosen IP is pinned in
`Target` and handed to the transport; redirects are refused (fail closed),
stricter than revalidating each hop. Rebinding is defeated because resolution
happens once and the pinned address is what the transport connects to.

## Validation

`cargo test --offline`: unit + fake-endpoint negative tests, real ES256 VAPID
sign/verify, RFC 8291 encryption round trip, SSRF negative matrix, replay /
backoff / re-read assertions, canonical 7 structural lint self-tests plus a
full-tree `source_lints::check`, `cargo fmt --all -- --check`, strict clippy
(`-D warnings -D clippy::correctness -D clippy::suspicious -D clippy::perf
-D clippy::complexity`). Nothing is deployed, merged or published.

Host: marceline-dev. Issue: `NHARAlNNTdqYZJHJaqdD8I-splash`.
