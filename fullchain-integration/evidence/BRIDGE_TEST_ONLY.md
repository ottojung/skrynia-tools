# External-edge sender bridge (test-only)

`fullchain-integration` gains one opt-in mode selected by `FC_SENDER_BRIDGE`
(pointing at a JSON config). It does **not** start a kernel or edge: the caller
owns a single disposable kernel/P0/P1 + trusted-edge stack, so the real sender
drains hints from the **same** P1 the browser later rereads.

The bridge constructs the real `webpush-sender` `HttpP1` against a loopback,
app-scoped edge capability, runs one durable-ledger pump cycle, delivers to a
controlled local HTTPS endpoint, verifies VAPID ES256 + RFC8291 AES-128-GCM,
and writes `plaintext_hex` (the exact decrypted 32-byte hint) to the configured
output path (must not exist; created exclusively with mode 0600). No operator/P0 token, no production endpoint, no DNS or real
credentials are involved. The normal `FC_ARTIFACTS` harness path is unchanged
when `FC_SENDER_BRIDGE` is absent.

Config fields: `edge_port`, `edge_host`, `edge_origin`, `edge_cap`,
`sender_cap` (64 hex), `ns` (64 hex), `key`, `ledger` (private dir), `out`.
The caller must have registered the scoped push grant/subscription/sender grant
on that P1 with the same `sender_cap`/`ns`/`key`, and must use the deterministic
`fixture()` subscription bytes from `scenarios/keys.rs` so the endpoint can
decrypt.

This is a test-only bridge, not a release feature and not an external push
provider. Every added Rust file stays within the repository's structural lint
limits (<=50 nonblank/noncomment lines per file, <=10 entries per directory).
