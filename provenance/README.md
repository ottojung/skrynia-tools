# Push-capable ordinary P1 WASM — source-pinned build + real-binary proof

Host `marceline-dev`, 2026-11-23. Owner-authorized bounded engineering.
**VERDICT: PASS (source/test/artifact), with a REQUIRED kernel companion fix.**
**NO production promotion, no merge, no real VAPID/subscriber/credentials.**
Generic kernel and externally supplied dumb Program0 business logic UNCHANGED.

## 1. Source pinned (never guessed)
- Repo: `https://github.com/ottojung/skrynia-tools.git`
- Exact SHA: `efe61173660b089df4e43dcd2840210d3070bfe2` (origin/pr65-head, DRAFT PR65,
  descendant of integration source `a4717fc8`). Verified with `git cat-file -t` after
  `git fetch origin`; detached worktree `/tmp/opencode/p1-push-build-efe6117`.
- `Cargo.lock` SHA256: `952cc5886f8151bbb499df1824d6196d4f7a420e75f83f1d5499a646fcd3368c`
- Required source facts confirmed at this exact SHA:
  1. `native-service/src/cb_guest/cap/disk_read.rs:18` = `call::call_with_capacity(&r, 8192 + 4)`
     (8196) — sized-read fix preventing code-7 on 256605-byte AppGet.
  2. `native-service/src/protocol/wire/mod.rs:25-32` = OP_PUSH_SUBSCRIBE 16,
     OP_PUSH_UNSUBSCRIBE 17, OP_PUSH_GRANT 18, OP_PUSH_SENDER_GRANT 19,
     OP_HINT_LIST 20, OP_HINT_READ 21, OP_HINT_ACK 22, OP_PUSH_SUB_FETCH 23.
  3. Full exact-key/subject-generation revocation + scoped sender implementation and
     T1-T8 unit tests (wrong grant, sibling key, ABA, seq reuse, rotate, legacy preserve).

## 2. Toolchain / build provenance (reused, not duplicated)
- rustc `1.93.0 (254b59607 2026-01-19)`, cargo `1.93.0 (083ac5135 2025-12-15)`.
- `RUSTC=/workspace/skrynia-buildstd-sysroot/bin/rustc` (sysroot wrapper rewrites
  `--print sysroot`; isolated rust-src at
  `/workspace/skrynia-buildstd-sysroot/lib/rustlib/src/rust`).
- Target: `wasm32-unknown-unknown` with `-Z build-std=std,panic_abort`. This is the
  pinned ABI target actually used by kernel PR120 / dumb P0 d57ab27 (exports
  `skrynia_main`, `__indirect_function_table`, imports `skrynia.read/write/...`);
  the task phrase "wasm32-wasip1" does not describe this established ABI.
- Linker `LLD 14.0.6` `/gnu/store/2a6hs7mwkl9abl9bxmm4akq2akq9b9ip-lld-14.0.6/bin/wasm-ld`.
- `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR=/workspace/skrynia-v3-integration-kit/wasm-target`
  (single shared wasm target; no duplicate huge Cargo target/cache created).
- Exact build command (also in `build.log`):
  `RUSTC=... RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR=... CARGO_BUILD_JOBS=2
   RUSTFLAGS="-C linker=.../wasm-ld -C link-arg=--export-table"
   cargo build --locked -Z build-std=std,panic_abort --target wasm32-unknown-unknown
   --release -p skrynia-native-service --features cb-guest`
- `--export-table` is required by the kernel callback ABI; it is supplied explicitly
  because `RUSTFLAGS` overrides the repo `.cargo/config` target rustflags.
- No private token or secret in any command, log, or artifact.

## 3. Artifact
- `skrynia_native_service_p1_push_efe6117.wasm`
- SHA256 `cf1322c3d41009c19c65771b353259b5f9c0d98157ec9075513edcd390c3477a`
- bytes `191755`
- Reproducibility: forced crate-only recompile from clean source reproduced the
  byte-identical SHA (`rebuild_determinism2.log`); PASS.
- Imports: `skrynia.eval_buffered`, `skrynia.write`, `skrynia.invoke`, `skrynia.read`.
  Exports: `memory`, `skrynia_mediator_cb`, `__indirect_function_table`,
  `skrynia_main`, `__data_end`, `__heap_base`.
- Push signatures present (`pinned_p1_push_opcode_audit_updated.py` /
  `push_opcode_audit.log`): `PUSH_PRESENT ... sigs=[hint, push, sub_fetch,
  subscribe, unsubscribe]`, VERDICT=PASS. All seven earlier pinned P1 artifacts
  remain NO_PUSH_SIGNATURES (the PR77 blocker).

## 4. Gates on exact source
- `cargo fmt --all -- --check`: PASS (FMT_RC=0).
- Strict Clippy `-D warnings -D clippy::correctness -D clippy::suspicious
  -D clippy::perf -D clippy::complexity`: PASS (CLIPPY_RC=0).
  (Bare group names are deprecated by this toolchain; qualified names used.)
- `cargo test --locked -p skrynia-native-service --lib`: 27 tests PASS, including
  25 push tests (t1-t8: exact key, wrong grant, sibling scope, revoke ABA,
  subject-generation rotation, seq reuse, legacy preservation, error-after-commit).
  Doc-tests not run: `rustdoc` binary is absent from this toolchain (environment
  limitation, not a source failure).

## 5. Real-binary E2E (actual WASM, no mocks)
Harness `push_e2e.py` (derived from the repo's proven `program0/integration/dumb_p0_http.py`):
real compiled kernel + external dumb P0 + this P1 over real HTTP, fake 32-byte
test bearer only, disposable temp dirs. No browser secret, no real VAPID.
- Kernel companion requirement discovered and proven:
  - Pinned PR120 kernel `3508938239c4b7f27591000e2d1d1d36f3b3cc1f976c470fe0e5f84274e3176d`
    + clean push P1: AppPut regresses with code 7 because generic
    `rust-server/src/fs/dir/path.rs::open_path` mapped every `openat` failure to
    ERR_IO(-5), so `disk.list()` on an absent nested `pushsub/.../subs` dir failed.
    RED evidence: `base_e2e.log`, `push_e2e.log`.
  - Owner's corrected generic-kernel PR125 exact head
    `856c029c6e428706a475b2fb06dbe2e2357eb105` (locally built binary SHA
    `4c6d34f93f59a5377e64fdeae319a200d358b7ca5290162ad0a6b618f62954dc`)
    maps ENOENT to ERR_NOT_FOUND(-2) only; RED/GREEN evidence:
    `kernel_PR125_build.log`, `push_e2e_PR125_cleanP1.log`.
  - Pinned PR120 binary restored to
    `/workspace/skrynia-v3-integration-kit/target/release/skrynia-server` (verified SHA).
- Dumb P0 `d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa` (unchanged).
- Echo guest `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5`.
- GREEN matrix with PR125 kernel (31 named PASS / ALL_OK=True, `push_e2e_PR125_cleanP1.log`, ALL_OK=True):
  POSITIVE op18 grant, op16 subscribe, AppPut hint, op19 sender grant, op20 list
  (seq=[1]), op21 read (32-byte new_hash match), op22 ack + idempotent replay,
  op20 empty-after-ack, op23 scoped sub-fetch, 256605-byte AppGet byte-exact;
  NEGATIVE wrong grant (9), sibling key (9), wrong sender cap (9),
  op23 wrong-sender endpoint read (9), unsupported opcode (1),
  malformed frame (1), wildcard key (9), revoke (op17) removes endpoint,
  revoke-ABA no resurrection, rotated subject revokes old endpoint,
  restart persistence of scope/subscription/256605 object.
- Fail-closed on genuine I/O: injected ENOTDIR at `pushsub` -> AppPut err 7,
  NOT silently treated as empty (`fault_inject_list.log`). No kernel/P0 business
  policy added to P1; no global -5 swallowing.

## 6. Decision / residual gates
- The P1 artifact PASS is coupled to the corrected generic kernel PR125 (generic
  FD errno semantics, independent review by
  `k1fDsFVOtllbVMXDUeNrNW-impulsive` ACTIVE, CI pending at posting). On unchanged
  PR120 the AppPut regression is a real blocker; do not promote.
- A local per-object sentinel P1 workaround was trialled and deliberately
  abandoned/deleted (never pushed) — see `abandoned_p1_workaround.md`.
- Trusted-edge / fake push endpoint: the separate PR77 loopback harness proves the
  sender side with a fake P1; chaining it to this real-binary P1 is a follow-up,
  not claimed here. No browser SW notification claim.
- Final: SOURCE GO, ARTIFACT GO, E2E GO (with PR125). MERGE/RELEASE/PRODUCTION NO-GO.
