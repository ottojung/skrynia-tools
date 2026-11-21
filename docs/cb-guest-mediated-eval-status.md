# cb-guest mediated nested-eval adaptation status

Branch: `agent/temperate-cb-guest-mediated-eval` (HEAD `aa6333a`, pushed) on tools PR #48 lineage (`agent/persist-svc-qualify-bonfires@282ed2b`, includes `7ca12bb`, `282ed2b`).
Kernel ABI references: nested mediation `6bc52ec/7695fd3/ccbbbdf` (`docs/nested-eval-abi.md`), buffered bridge `004c01b` on `agent/hQGjVJwW8gkAfcPE3WnB1s/8dWY-buffered-eval` (`docs/buffered-eval-abi.md`, stacked atop nested reentrancy/PR117 lineage).

## Adapted (this branch)

- `program0/src/proxy/abi.rs`, `program0/src/proxy/entry.rs`: P0 forwards service via `skrynia.eval_delegate` (generic capability delegation) so forwarded P1 may act as immediate mediator with eval capability. P0 remains token check/strip + opaque forward only (no app/business/scoping semantics, no special Program1).
- `native-service/src/cb_guest/eval/{mod,cb,kernel}.rs`: mediator callback (`skrynia_mediator_cb`, indirect-table index) mediates P2 `invoke` requests through `appdata::cb::guest_callback` under app-scoped delegated rights (self invoke_id namespace only). `MediatorEval` calls kernel `skrynia.eval_buffered(code_ptr,code_len,req_ptr,req_len,res_ptr,res_cap,cb_idx)` (004c01b ABI) with immediate-parent callback mediation, returning buffered P2 output bytes for registry `Request::Invoke`.
- `native-service/src/cb_guest/entry.rs`: service uses `MediatorEval`.
- `native-service/src/registry/reg_ops/invoke.rs`: sets/clears cb-guest mediation context for raw kernel callback path (host `root_eval` hooks gated out for cb-guest).
- `native-service/src/cb_guest/fd.rs` (`aa6333a`): read/write `skrynia_main` input/output FDs via direct kernel stream imports `skrynia.read/write` (not P0 relay), since transferred guest FDs are inaccessible in provider callback context. This fixes guest-side service I/O failure/empty-body 500 symptoms on compatible runtimes.
- `native-service/src/cb_guest/cap/wire.rs` and `program0/src/proxy/host/wire.rs` unchanged: P0↔P1 relay opcodes 1-13 are `OPEN_DISK..TRY_LOCK,RELEASE_LOCK`; nested/buffered eval are direct kernel imports, not relay opcodes. No opcode-13 conflict in relay wire.

## Host validation (independent target `/tmp/opencode/target-bonfires`, `aa6333a`)

- `cargo fmt --all -- --check`: PASS
- `rustc --test source_lints/mod.rs -o /tmp/opencode/skrynia-lint-tests-bonfires`: PASS (7/7)
- `cargo clippy -p skrynia-native-service --all-targets --locked -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity`: PASS
- `cargo test -p skrynia-native-service --locked`: PASS (2 unit + 35 integration, 0 failed)
- `cargo test -p skrynia-native-service --doc`: **not run** (`rustdoc` missing in environment)

## Exact wasm artifacts (HEAD `aa6333a`, rebuilt 2026-11-21)

Procedure per `/workspace/skrynia-v3-integration-kit/wasm-build-record/README.md` (host `rustc 1.93.0`, `cargo 1.93.0`, `wasm-ld 14.0.6` `/gnu/store/2a6hs7mwkl9abl9bxmm4akq2akq9b9ip-lld-14.0.6/bin/wasm-ld`, sysroot snapshot `/tmp/opencode/rust-src-snap-1.93.0`, `RUSTC_BOOTSTRAP=1`, `-Z build-std=std,panic_abort`, `--target wasm32-unknown-unknown --release`, `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR=/workspace/skrynia-v3-integration-kit/wasm-target`):

- `skrynia_program0.wasm` (p `skrynia-program0`): **38819 bytes**, sha256 **`65396f6105f907dbb95898a483a0b93ea070b74ac316d5a17fee7c4e6433abf7`** (imports `skrynia.eval_delegate`; relay wire 1-13 matches cb-guest disk/cap protocol). Matches prebuilt 006a0d3 P0 hash noted in kit record.
- `skrynia_native_service.wasm` (`-p skrynia-native-service --features cb-guest`): **133101 bytes**, sha256 **`882fe38434d136b9434ace26e987c2caa324583cfa9a3200262cbaddb17fd562`** (imports: `skrynia.read`, `skrynia.write`, `skrynia.eval_buffered`, `skrynia.invoke`).

Note: an intermediate/stale wasm-target build produced `e904f02d6f191dc4fd68d19fa4ceea04dadffa2501d595b2e5549c38370fd4b4` (130194 B, stub/no `eval_buffered` import) from a diagnostic source state; it was superseded by a clean rebuild from HEAD `aa6333a` yielding `882fe384...` above.

## Runtime diagnosis (no kernel builds)

- Kernel (buffered-eval 004c01b lineage, 8dWY worktree): `/tmp/opencode/skrynia-kernel-8dWY-hQGjVJwW8gkAfcPE3WnB1s/target/debug/skrynia-server`, sha256 `afa2861259b57875cb2cf7b086b840d0fe3cd91b7445dbb665f499c08c6802fd`.
- Real-triad P0 (delegating, provenance unconfirmed): `/tmp/opencode/skrynia-real-triad/p0/target/wasm32-unknown-unknown/release/skrynia_program0.wasm`, sha256 `06dac10cbb06dafc85c42a8e7af93d9bd859b6c1b62c864ad0082e9c27722518` (imports `eval`, `eval_delegate`, etc.; relay wire differs from cb-guest/65396f61: e.g. OPEN_DIRECTORY/SYNC/DELETE/RENAME/RANDOM/EVAL opcodes differ, has no lock ops). **Not relay-compatible** with cb-guest disk mediation.
- With P0 `06dac10c` + P1 `882fe384` (and earlier `e904f02d`) on kernel `afa28612`, cb-guest returned correct HTTP 200 protocol responses for valid native requests: empty payload -> `01010100000001` (decode err 1), `ListPrograms [1,14,0,10,0]` -> `01010100000007` (store/dir err, expected given relay incompatibility preventing disk ops), `GetActive`/`Lookup` unknown -> `01010100000008` (missing). Confirms stream I/O fix and protocol framing over real POST+P0+P1 path.
- Compiled P0 `65396f61` as root returned HTTP 500 (empty body) for echo guest and cb-guest on kernels `afa28612` (8dWY) and `c86143182890b4808e6f002f3fc73a6e16cba804793512af6ec61aceb7d3f0b5` (temperate), despite matching cb-guest relay wire and importing supported host funcs; root-cause not isolated here (no kernel debug/log instrumentation). Real-triad P0 `06dac10c` echoed OK and framed cb-guest protocol on `afa28612`.

## Remaining blockers / coordination (for H5, wB1, hQGj)

1. **Pinned compatible runtime for full mediated proofs:** need a paired kernel + P0 where kernel implements `eval_buffered` and immediate-parent callback mediation (004c01b/8dWY lineage) and P0 performs opaque token-strip/`eval_delegate` forward **with relay wire compatible with cb-guest/65396f61** (OPEN_DISK..SYNC,RANDOM,TRY_LOCK,RELEASE_LOCK). P0 `65396f61` matches relay protocol but failed as root on tested kernels; P0 `06dac10c` delegated on `afa28612` but its relay wire is incompatible with cb-guest disk ops (stateful ops fail with store errors). No authoritative pinned pairing available in this environment.
2. **End-to-end adversarial proofs pending compatible runtime:** real HTTP POST+P0+P1+P2 validation of authorized stored P2 execution with P1-mediated callbacks + app-scoped appdata create/read/write, invalid/sibling/other-namespace/root denial, nested/trap/cancel/restart cleanup and liveness, and A-B-A CAS code-only rollback (stage/seal/activate revision protocol) could not be completed without a working disk-relay + eval-delegate pairing. Host-level coverage exists (35 integration tests incl. releases/ABA/CAS, lock safety, durable replace/recover) but is not a substitute for compiled wasm32/HTTP mediation proof.
3. **Doc-tests:** `cargo test -p skrynia-native-service --doc` not run (`rustdoc` missing).
4. **No production, DNS/TLS, real creds/secrets, CLI, or Ivanka modifications. No kernel builds/duplicate kernel caches.** Board/Antonina posting not possible from environment; handoff via this status, branch, and draft PR.
