# Native v3 cross-namespace Wake full-chain acceptance

Status: **compiled disposable integration PASS**, not real browser/provider,
production deployment or any source of private credentials.

## Exact SHA-pinned runtime

- Skrynia v3 kernel: ottojung/Skrynia main commit
  1c5639debea8838277e5f22d9136ff263e894071, artifact
  kernel_v3_main_1c5639d SHA256
  ac301e18fcabf2cbf9f246fbaf1b7985e4062b6225da125f873138a2ed77401c.
- Native P1: ottojung/skrynia-tools head
  18f9123a55070118e4f2fb9aa2c09ad6d16690b2 (PR81, merged to
  PR80 development branch), --features cb-guest built for
  wasm32-unknown-unknown; artifact p1_wake_18f9123a.wasm
  SHA256 d47f824f6c3d1630e67bba93ff2b6c829b9c685d12e17fe2b765fccd668a595b.
- Dumb user-supplied P0, trusted edge and P2 echo guest: SHA256 values
  in fullchain-integration/src/chain/pins/table.rs. All five artifacts
  must be separately built or supplied in artifacts/, **not committed**;
  runtime refuses any missing or mismatched bytes before spawning anything.

## Reproduce using existing build dependencies

With the exact source commits, build Rust 1.93 kernel using
cargo build --offline --locked --release -p skrynia-server. For P1:
RUSTC_BOOTSTRAP=1 cargo build --locked -Z build-std=std,panic_abort
--target wasm32-unknown-unknown --release -p skrynia-native-service
--features cb-guest, with the **matching** Rust 1.93 rust-src sysroot,
wasm-ld linker and -C link-arg=--export-table. Never replace the
pinned P1 with an older artifact lacking wake opcodes 24/25/26.

Run: cargo run --offline --locked --manifest-path
fullchain-integration/Cargo.toml --bin fullchain.
The harness uses independent disposable kernel/edge/provider data trees
and test-only deterministic caps. It prints a checked SHA256 manifest,
existing sender/VAPID/HTTPS delivery tests, then five extra Wake verdicts,
finally FULLCHAIN_OK if and only if every assertion succeeds.

## Proven real operations, not a mock

- Three independently registered P1 namespaces (owner, writer A, writer B);
  one reserved exact skrynia-wake-v1 subscription; per-source Wake-only
  grants op24; disjoint source updates op25.
- Sender hint_list/read op20/21 sees g1; writer B advances to g2;
  stale op22 ACK(g1) **cannot clear** g2; wrong-source grant and
  arbitrary-wake-key requests fail.
- Real sender sends g2 to local HTTPS provider, which verifies ES256 VAPID
  and decrypts AES128GCM opaque invalidation. Sender clears exactly g2.
- 1040 further op25 writes without a sender ACK, alternating the two
  namespaces; they ALL commit, and one fixed-slot hint with generation
  1042 remains. ACK(1042) clears it. op26 DeleteWake produces g1043.
- Existing full-chain suite still tests negative host/origin/caps,
  subscriber revoke, sender retry/backoff, restart and duplicate prevention.

Evidence from actual first integrated 80.55s run is available in the
operator's dated test log (not a release binary). This harness is a
*separate* acceptance binary, not included in the rapid make test
unit suite. It writes to disposable temporary paths only.

**Remaining gates:** browser PushManager on real HTTPS origin, external
provider push to a real browser subscription, service worker authoritative
reread, grant rotation/revocation, five-app backed-up data migration,
operator credentials and TLS/DNS/cutover approval.
