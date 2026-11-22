# Native-v3 P1 push: narrow grant + least-privilege sender (source-only fix)

Owner: agent-qNECjyPbVXQSxEE051NGjC-screaming
Host: marceline-dev
Base: `skrynia-tools` P1 draft `feat/p1-push-subscribe-change-hints` @
`a4717fc8fb0a39f7cab3b0bb09ac5751545b06f3` (pre-existing, unqualified, do NOT merge).
Branch: `antonina/p1-push-grant-sender-qNECjyPbVXQSxEE051NGjC`.
Scope: SOURCE EDIT + SELF-REVIEW ONLY. No Cargo/WASM/Podman build, no CI, no
deploy, no PR/merge, no real VAPID/subscriber data, no DNS/TLS/secret changes.

## Defect 1 — bare namespace write_cap was subscribe authority → FIXED

The draft let any holder of a 32-byte namespace `write_cap` subscribe any valid
exact key (`a4717fc8:registry/appdata/push/subscribe.rs:21,30`), with no grant
record anywhere in `native-service/`. Replaced with an explicit narrow grant.

- `registry/appdata/push/grant/fmt.rs:6` — `Kind::{Subscribe,Sender}`;
  `:14-15` separate durable subtrees `pushgrant/` and `pushsender/`;
  `:20` `grant_path(base, kind, grant_ns, key)` keys the record by
  `grant_ns = SHA256(grant_cap)` and `SHA256(key)`;
  `:30-36` body `namespace || key_len || key || subject`;
  `:39-50` `decode` accepts a record only when stored key AND subject equal the
  presented ones, returning the granted namespace.
- `registry/appdata/push/grant/issue.rs:9-23` — `issue` is authorized only by a
  32-byte namespace `write_cap`; it derives `ns = SHA256(write_cap)` (read cap)
  and writes the exact-key record. A `grant_cap` alone cannot issue anything.
- `registry/appdata/push/grant/verify.rs:9-25` — `verify` reads the record at
  `SHA256(grant_cap)`/`SHA256(key)`; absent or mismatched record => `BadAuth`.
- `registry/appdata/push/subscribe.rs:14,18,32-39` — `subscribe` now takes a
  **grant_cap** (not a write_cap) and calls `verify` before any state change.
  `unsubscribe.rs:11,25-32` likewise. Exact-key grammar still enforced by
  `registry/appdata/path.rs:12-21` (prefix/wildcard/empty/malformed rejected).
- Wire/registry plumbing: `protocol/wire/mod.rs:27` `OP_PUSH_GRANT=18`,
  `wire/req.rs:43` `PushGrant`, `reg_ops/push_grant.rs:10-31`
  `Registry::push_grant` (writer-only issuance, subscriber bound via
  `SHA256(subscriber)` as subject).

Result: possession of a broad write cap is no longer subscribe authority; a
narrow grant is durable, exact-key-bound, subscriber-bound and revocable by
deleting its record. It cannot write objects or mint root tokens.

## Defect 2 — no least-privilege sender identity / hint list-read-ack → FIXED

- Separate identity: `sender_cap` is a 32-byte bearer capability distinct from
  any write grant or root P0 token. Scope issued only by a namespace writer via
  `protocol/wire/mod.rs:28` `OP_PUSH_SENDER_GRANT=19`, `wire/req.rs:44`,
  `reg_ops/push_grant.rs:34-51` `Registry::push_sender_grant`, using
  `Kind::Sender` (`grant/fmt.rs:15`), subject `[0u8;32]`.
- Scope check: `push/sender/scope.rs:5-19` — verifies a `Kind::Sender` grant for
  `(SHA256(sender_cap), ns, key)` and requires the granted namespace to equal
  the requested one; no broad raw-FS hint access.
- Scoped ops:
  `push/sender/list.rs:4-27` `hint_list` (sorted seq list, `MAX_HINTS=1024` cap,
  body = u32 count + u64 LE seqs);
  `push/sender/read.rs:4-20` `hint_read` (exactly 32 opaque bytes, never data);
  `push/sender/read.rs:24-37` `hint_ack` (idempotent replay; real remove errors
  propagate).
- Wire/registry: `wire/mod.rs:29-31` opcodes 20/21/22, `wire/req.rs:45-47`,
  `decoder/ops/push.rs` parse, `wenc_req/encode_sender.rs:11+` encode,
  `protocol/ops/sender.rs` handle, `reg_ops/hint_ops.rs:9,20,32` facades,
  `protocol/ops/push.rs:13-18` dispatch. Sender has no app registry, no business
  policy and no Register/Set/Revoke path.

## Additional fixes found during self-review

- `unsubscribe.rs:35-38` — the draft did `let _ = disk.remove(&file)` and always
  returned `Ok`, swallowing disk failures. Now missing => `PushErr::Missing`,
  and remove errors are mapped to `PushErr::Disk`.
- The pinned draft did not type-check: `protocol/ops/push.rs` passed a `RegErr`
  into a `push_err(PushErr)` function. Replaced with the crate's `map_res`
  (`protocol/ops/push.rs:2,20-39`). Also removed a duplicate `Disk` import in
  `registry/reg_ops/mod.rs:10-11`.

## Red-green test outlines T1-T5 (written, not run)

`registry/appdata/push_tests/` (fake `MemDisk` + `FailRemove`, no ambient I/O):

- T1 `t1.rs:5` exact-key grammar negatives (wildcard/prefix/empty/leading-dot/
  slash/double-slash/NUL) and `t1.rs:24` foreign writer cannot reuse a grant.
- T2 `t2.rs:4` durable grant/subscribe, reopen on the same disk, re-subscribe,
  unsubscribe and missing-unsubscribe error.
- T3 `t3.rs:5` no broad cap, no grant, wrong subscriber, wrong key and malformed
  key can subscribe.
- T4 `t4.rs:6` sender scope is exact (foreign sender/ns/key denied), hint list
  body shape, 32-byte read, idempotent ack.
- T5 `t5.rs:6` only a writer issues grants; injected remove failure surfaces on
  unsubscribe.

## Validation performed (compile-free)

- `rustfmt --edition 2024 --check` on every changed/new `.rs` file: clean.
- Compile-free replica of `source_lints` (code LOC <= 50 per file, <= 10
  direct entries per folder, no `super::super` / nested module imports):
  ALL CHECKS PASS over `native-service/src`.
- Source-static review of file:line evidence above.
- NOT run (fail closed, per task): `cargo build/test/clippy`, WASM/Podman,
  CI, browser/fake-push-endpoint/VAPID acceptance, merge/deploy.

## Residual work (not claimed)

1. Build/CI green at a new pinned SHA; run T1-T5 for real.
2. Implement the external outbound sender service (HTTPS WebPush, real
   VAPID-format auth, DNS/rebinding/private-IP/redirect/port controls, backoff)
   using these scoped ops; it is deliberately not in this P1 patch.
3. Decide whether the sender also needs a scoped subscription-endpoint read op
   for delivery (currently only hint queue ops are provided).
4. Five-app integration off legacy push, full CI and independent pinned-SHA
   review, maintainer merge. Keep the issue OPEN.
