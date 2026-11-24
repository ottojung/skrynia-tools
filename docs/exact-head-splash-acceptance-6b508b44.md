# Exact-head disposable acceptance evidence — native v3 Web Push (`#splash`)

Status: **development/disposable acceptance PASS at exact current heads.** This
is NOT a release GO. No production, real subscriber data, real VAPID signing
secret, DNS/TLS cutover or browser PushManager subscription with an external
push service is involved. Whole release remains NO-GO pending the operator-held
gates listed at the end.

Owner: agent `XGxDTWy9OdGuqE0FLcvVRe-diaphysis`, host `marceline-dev`.

## Exact source pins (verified, not trusted from notes)

| role | repository | exact head |
|---|---|---|
| tools | `https://github.com/ottojung/skrynia-tools.git` | `6b508b4444b255412e6701b454035af3e8c41247` |
| kernel | `https://github.com/ottojung/Skrynia.git` | `806c172402880389eee94162a020c0f9c4c1bff2` |
| apps (Along / hingelike) | `https://github.com/ottojung/skrynia-apps.git` | `a2c87c6c8294c885147b4c304994ac5570437998` |
| toolchain | rustc/cargo | `1.93.0` |
| Chromium | local binary | `147.0.7727.137` |
| playwright-core | installed module | `1.65.1` |

The tools head `6b508b44` differs from the exact-kit source pin `73d8fa1` only
in CI/workflow and web-admin test files; `v3-trusted-edge-gateway`,
`webpush-sender`, `native-service`, `program0`, `Cargo.toml` and `Cargo.lock`
source trees are byte-identical (confirmed by the lS8KU07 commotion kit
manifest). No kernel/guest/edge rebuild was performed.

## Exact artifact kit (reused, not rebuilt)

Artifacts and their authoritative digests come from the commotion shared kit
`/workspace/skrynia-current-kit-lS8KU07-20261124/COMMIT_MANIFEST.txt` and the
apps manifest `/workspace/skrynia-commotion-lS8KU07EJvFcSpHg1sUlEz-20261124/apps/evidence/current-stack-manifest.json`.
All five digests were re-verified from bytes with `sha256sum -c` before use.

| harness name | source path | SHA-256 |
|---|---|---|
| `kernel` | `skrynia-exact-kit-73d8fa1/kernel-release` | `a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1` |
| `root.wasm` (dumb P0) | `skrynia-exact-kit-73d8fa1/root.wasm` | `44e845a4ca1bfcf6713078ad40d2ce7d387595b08351a5a882bbc38549780c95` |
| `service.wasm` (ordinary P1) | `skrynia-exact-kit-73d8fa1/service.wasm` | `c7239fc5bfd6977e7835eef461a75e274270d9e721d8a2d88121906b1b0c3a36` |
| `edge` (trusted gateway) | `skrynia-current-kit-lS8KU07-20261124/edge` | `dc1c91e80a3e5162d8b08d9d43f665e240eb4314e7e1d09dcda9d29f309829de` |
| `echo.wasm` | `skrynia-v3-integration-kit/tests/guests/echo_guest.wasm` | `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5` |

## What was actually run

1. Fresh isolated clone of tools/apps into a unique agent-owned directory;
   branches created from the exact heads above. No shared checkout edited.
2. `FC_ARTIFACTS` assembled with the five named files and an exact five-entry
   lowercase `SHA256SUMS`; the harness verified all five digests before spawning
   any process (`pin_verify=ok`).
3. `fullchain-integration` harness built at exact head with the cached Cargo
   target directory (`CARGO_BUILD_JOBS=2`, `--offline --locked`, 5 s, no
   kernel/WASM/edge rebuild), then run once against the compiled stack.

Durable stdout: `evidence/EXACT_HEAD_6b508b44_FULLCHAIN.log`,
sha256 `6ade46515179bfced76a138a373f60426bd7f7426b6cd496b4a0ba49f64b4247`,
exit code 0, ending `FULLCHAIN_OK`.

4. The real Along service worker (`public/sw.js`, sha256
   `b1c9135be629bf0bdee47fbb247b9921f846090590f2d9d3105f9e46e8b23739`) was
   driven in real Chromium 147 via CDP
   `ServiceWorker.deliverPushMessage` by
   `apps/source/hingelike/scripts/test-along-sw-push-chromium-cdp.mjs`
   (test sha256 `b3508f867c656ce1bf2190db62773cb60c8b3fc0957b977748fad0f0462766c3`).

Durable stdout: `evidence/EXACT_HEAD_6b508b44_ALONG_CDP.log`,
sha256 `92772e2c8eeef053b6dc0a2f4d819f6ae3c036cd469f02fa1b3b9a5ae0830b79`,
exit code 0, `PASS real Chromium CDP push: 13 browser-handled gates`.

5. Crate gates at exact head (cached target, offline): `cargo test --offline
   --locked --all-targets` for `webpush-sender` (all binaries, including
   delivery/retry/negative/negative2/residuals/ssrf/vapid/wire, 7 tests in each
   of the final binaries) and `fullchain-integration` (7 unit tests including
   correct-pin accept, wrong-pin/tampered-bytes reject and runtime manifest
   computation); `cargo fmt --all -- --check` clean. All exit 0.

## Results — compiled stack + isolated sender + controlled HTTPS endpoint

The single harness run exercised the real generic kernel, dumb P0, ordinary
P1 WASM, trusted edge, real HTTPS push sender and a disposable local HTTPS
push provider. Key evidence lines:

- `pin_verify=ok` and all five SHA-256 lines (above).
- `vapid_es256_verify=ok`, `aes128gcm_decrypt=ok`, `push_seen=1`,
  `vapid_sub=mailto:ops@example.invalid` (synthetic subject, no real credential).
- `restart_no_duplicate=true`, `revoke_stops_delivery=true`,
  `hint_list_empty=true` (replay/ack/idempotence).
- Negative/SSRF: `neg_wrong_grant=9`, `neg_sibling_key=9`, `neg_bad_sender=9`,
  `neg_bad_fetch=9`, `neg_malformed=1`, `neg_wrong_version=1`,
  `neg_edge_bad_cap=401`, `neg_edge_bad_host=403`, `neg_edge_bad_origin=403`,
  `neg_redirect_err=Transport("status")/seen=1` (redirect not followed),
  `neg_backoff_attempts=Exhausted/seen=3`.
- Durable wake/ordinary coverage: `wake_crossns=two_writers`,
  `wake_stale_ack=preserves_generation`,
  `wake_provider_vapid_and_decrypt=ok`, `wake_offline_1040_writes=ok`,
  `wake_delete=advances_generation`, `ordinary_hint_budget=256`,
  `ordinary_fallback_highbit=real_wasm`,
  `ordinary_fallback_vapid_decrypt=ok`,
  `ordinary_fallback_sender_ack=ok`.

Exact-key rejection (no prefix/wildcard/empty/foreign-scope) is covered by the
P1 negatives above and by the crate suites; no kernel/P0 source change was made
and no kernel subscription table, app ID, endpoint/policy, SSRF exception or
VAPID secret exists in the kernel or P0.

## Results — real browser worker with a controlled test transport

The CDP run proved 13 browser-handled gates on the actual `public/sw.js`:
same-origin provisioning over `MessageChannel`, malformed-hint suppression,
binary AppGet-only reread, fail-closed missing state, one-shot match/message
notification with baseline advanced after notification, dedupe, self-message
suppression, revocation/unknown-identity refusal, fingerprint rotation, real
Chromium restart persistence, post-restart one-shot wake, and transport hygiene
(app-scoped edge capability only; no token or key material in the worker).

The worker treats the payload as an **opaque 32-byte hint only** — identity,
channel, match and message meaning come solely from the authoritative native
reread, never from the pushed bytes.

## Honest claim boundary (do not overstate)

| claim | status |
|---|---|
| Compiled kernel + dumb P0 + ordinary P1 + trusted edge + isolated sender | PASS (disposable, exact head) |
| Real VAPID-format ES256 signature and AES-128-GCM decryption at controlled HTTPS endpoint | PASS (synthetic key/sub only) |
| Restart, retry/backoff, revocation, SSRF/redirect/host/origin negatives | PASS (disposable) |
| Real Chromium service worker driven by received opaque hint via CDP bridge | PASS (browser-handled) |
| External Web Push provider delivery to a real browser subscription | NOT TESTED — operator gate |
| Browser `PushManager.subscribe` with real credentials/VAPID public key | NOT TESTED — operator gate |
| Production TLS/DNS/cutover, subscriber data, five-app backed-up migration | NOT TESTED — operator gate |
| Whole v3 release GO | NO-GO |

The CDP bridge is a **controlled test transport** (`ServiceWorker.deliverPushMessage`
with a 32-byte raw string payload) and an in-process binary AppGet fixture. It
is explicitly NOT an external push service, NOT a browser subscription and NOT
compiled-stack delivery to the worker. The compiled-stack run and the browser
run are separate exact-head runs in this environment; this document does not
claim they were one continuous cross-process pipeline.

## Coordination / scope discipline

- Reused the lS8KU07 commotion published edge and exact-current kit; did not
  rebuild kernel/WASM/edge or delete caches. `editor` / wake reliability work
  (5zNq, `SR2hUr`) was not touched; no wake fix duplicated or weakened.
- No production/VAPID secrets, no real subscriber data, no Ivanka/MiCall, no
  shared checkout edits, no legacy `GET /_skrynia/push/vapid` or store fallback
  introduced. No SSRF or kernel limit weakened.

## Remaining operator gates (unchanged, release stays NO-GO)

1. Real external push provider + real Chromium `PushManager` subscription and
   notification delivery with operator-held credentials/TLS/DNS.
2. Trusted-edge active-state verification path for browser subscription status.
3. Five-app backed-up migration/cutover and whole-release sign-off with
   independently pinned SHA review.

Reproduce locally with the documented commands and the reusable artifact
manifest above. `@ottojung`
