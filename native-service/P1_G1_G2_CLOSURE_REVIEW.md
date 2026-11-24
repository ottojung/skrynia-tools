# Native-v3 P1 Web Push — G1/G2 closure review

Owner: agent-yExng0nkivViAifHoSUnIx-resented. Host: marceline-dev.
Base: skrynia-tools a4717fc8fb0a39f7cab3b0bb09ac5751545b06f3 (pinned draft).
Combined: qNEC fix 5f9abcf (patch f0e4975…) + G1 corrective patch 6c76fef… + G2/compile closure.

## Per-gap disposition

- G1 (RED supplied tests): FIXED. Applied the corrective test patch; grant subject in
  t2.rs, t5.rs and support/seed.rs is now `crate::hash_bytes(&subscriber)` =
  f0e38b830ebd8a506615ecd154330ec07ff6bf5030447b44e297db1d4b7514ac, matching
  subscribe.rs's `hash_bytes(subscriber)` verification. Red-green demonstrated with a
  scratch worktree: restoring `[1u8; 32]` makes T2/T4/T5/T6 fail (4 failed), the fix makes
  them pass.
- G2 (no scoped subscription-endpoint read / delivery path): FIXED. Added
  OP_PUSH_SUB_FETCH=23, Request::SubFetch, decoder/encoder/dispatch plumbing and a
  Registry::sub_fetch facade backed by `push/sender/sub_fetch.rs`. It calls the existing
  `scope()` check (Kind::Sender grant bound to one exact (ns,key)), returns only the
  bounded stored subscription blobs (u32 count + [u32 len + bytes]*) to the scoped sender,
  and never exposes them to a browser/public caller. T6 covers exact scope, foreign cap,
  wrong ns/key and a wire round-trip. Hint ops remain opaque (32-byte hash) and unchanged.
- Additional blocking gap found by actually compiling (all earlier work was compile-free):
  the pinned base a4717fc did NOT type-check (8 lib errors), and the qNEC patch added
  further errors. FIXED mechanically (import re-exports in protocol/ops/mod.rs, grant
  id/verify_grant re-exports, test visibility, `push_grant` lock param removal,
  `push_subscribe` Vec→() mapping, test turbofish annotations). No logic/policy change.

## Validation actually run

- `rustfmt --edition 2024 --check` on every changed/new .rs: ALL CLEAN.
- Compile-free source_lints replica (`p1-fix-lintcheck.py`): ALL STRUCTURAL CHECKS PASS
  (<=50 code LOC/file, <=10 dir entries, no super::super or nested module imports).
- `cargo test -p skrynia-native-service --offline --lib --tests`: 10 unit + 35 integration
  PASS, 0 failed (CARGO_RC=0). Reused cached sha2 deps; no new heavy/WASM/Podman build.
- `cargo test --doc` NOT runnable: `rustdoc` binary is absent in this environment
  (environment limitation, not a code failure). No full CI, WASM/Podman, browser/fake
  endpoint/VAPID acceptance. Disk /dev/loop9 100% used (~1.8G free), below founder floor.

## Scope / non-goals

No kernel, Program0, legacy PR #123, main, CI, DNS/TLS, real secrets, deploy or merge.
The external VAPID/HTTPS sender service, five-app migration, full-stack acceptance and
independent pinned-SHA review remain release gates. Issue stays OPEN; no merge claimed.
