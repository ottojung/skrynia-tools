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

## Runtime

`src/main.rs` is a minimal operator-run standalone sender process. It uses
production-only transport/capabilities: `HttpsTransport::production()` over
`HttpP1`, `FileLedgerStore`, `OsResolver` (DNS), `OsClock`, and `OsRng`
(`/dev/urandom`). The test transport is feature-gated (`test-transport`) and is
not used by the executable.

### Configuration

The process reads config from the `WEBPUSH_SENDER_CONFIG` environment variable,
which must name a local JSON file. The config file must be a regular,
non-symlink file with mode `0600` (no group/other bits) and size <= 64 KiB.
Secrets are not formatted by `RuntimeConfig` or logged by the runtime; config
and fatal errors are logged as redacted static class labels only.

Required JSON fields:

- `edge`: `{ ip, port, host, origin, capability, timeout_ms, max_request_bytes, max_response_bytes }`
  for the scoped P1 HTTP edge. `edge` is validated by `Edge::validate()`.
- Pump/config: `sender_cap`, `ns`, `key`, `max_payload`, `max_attempts`,
  `base_backoff_ms`, `max_backoff_ms`, `ttl_secs`, `topic`, `vapid_sub`,
  `ledger_epoch` (validated by `Config::validate()`).
- `vapid_private_key`: 32-byte hex ES256 private key (server-side only).
- `ledger_dir`: private single-writer directory for durable ledger files.
- `poll_ms`: bounded poll interval, `250..=300000`.

Hex fields are hex-encoded bytes (`sender_cap`/`ns`/`vapid_private_key`: 32
bytes, `topic`: 16 bytes). Do not commit or log real operational config.

### Single-writer lock and ledger directory

Before entering the loop, runtime validates `ledger_dir` is a directory with no
group/other mode bits. It acquires a persistent lock inode
`<ledger_dir>/sender.lock` with `File::try_lock()` and holds it for process
lifetime. It does not delete stale lock files. Ledger reads are bounded
(`take(MAX_LEDGER_BYTES+1)`), reject non-regular/symlink/FIFO files, and temp
commit writes use exclusive create with mode `0600`; dir-fsync failure after
rename is treated as uncertain durability.

### Disposable container qualification

Run qualification in a disposable container/VM with synthetic config only. Do
not mount or pass real operational credentials, production endpoints, or real
VAPID keys. Tests must not contact external endpoints.

Example (Podman, disposable, no network needed for tests):

```sh
# from repo root, disposable container with Rust toolchain
podman run --rm -it --network=none \
  -v "$PWD":/src:ro -w /src/webpush-sender \
  rust:1.93-bookworm \
  cargo test --offline --lib --tests
```

For a local smoke run against a disposable test harness (not production), create
a private temp directory and a synthetic config file:

```sh
umask 077
tmp=$(mktemp -d)
mkdir -p "$tmp/ledgers"
chmod 0700 "$tmp/ledgers"
cat > "$tmp/config.json" <<'JSON'
{ "...synthetic fields only..." }
JSON
chmod 0600 "$tmp/config.json"
WEBPUSH_SENDER_CONFIG="$tmp/config.json" cargo run --offline
```

The example config must be filled with disposable/synthetic values and a test
edge that is locally controlled; never use real production secrets or public
push endpoints during qualification.

## Validation

`cargo test --offline --lib --tests`: unit + fake-endpoint negative tests, real
ES256 VAPID sign/verify, RFC 8291 encryption round trip, SSRF negative matrix,
replay/backoff/re-read assertions, runtime config/lock/dir redaction checks, and
structural lint self-tests. `cargo fmt --all -- --check` and strict clippy
(`-D warnings -D clippy::correctness -D clippy::suspicious -D clippy::perf
-D clippy::complexity`) should pass. If clippy build scripts fail to load
`libgcc_s.so.1` in a Guix/rustup environment, set
`LD_LIBRARY_PATH` to include a directory providing `libgcc_s.so.1` (for example
`/usr/lib/x86_64-linux-gnu`) for the cargo invocation only.

Nothing is deployed, merged or published. No real operational configuration is
used by tests.

Host: marceline-dev. Issue: `NHARAlNNTdqYZJHJaqdD8I-splash`.
