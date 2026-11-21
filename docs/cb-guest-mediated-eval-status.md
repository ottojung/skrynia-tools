# cb-guest mediated nested-eval adaptation status

Branch: `agent/temperate-cb-guest-mediated-eval` (based on PR #48 `agent/persist-svc-qualify-bonfires@282ed2b`).
Kernel ABI references: nested mediation `6bc52ec/7695fd3/ccbbbdf` (`docs/nested-eval-abi.md`), buffered bridge `004c01b` on `agent/hQGjVJwW8gkAfcPE3WnB1s/8dWY-buffered-eval` (`docs/buffered-eval-abi.md`, stacked atop nested reentrancy/PR117 lineage).

## Adapted in this branch

- `program0/src/proxy/abi.rs`, `program0/src/proxy/entry.rs`: P0 forwards service via
  `skrynia.eval_delegate` (generic capability delegation) instead of plain `eval`, so the
  forwarded P1 may act as an immediate mediator with eval capability. P0 remains token
  check/strip + opaque forward only (no app/business/scoping semantics).
- `native-service/src/cb_guest/eval/{mod,cb,kernel}.rs`: added cb-guest mediator callback
  (`skrynia_mediator_cb`, indirect-table index) that mediates P2 `invoke` requests through
  `appdata::cb::guest_callback` under app-scoped delegated rights (self-namespace/invoke_id
  only). `MediatorEval` now invokes kernel `skrynia.eval_buffered(code_ptr,code_len,req_ptr,req_len,res_ptr,res_cap,cb_idx)`
  (004c01b ABI) with immediate-parent callback mediation, returning buffered P2 output bytes
  for registry `Request::Invoke` (replaces `NoEval` / prior `-4` bridge stub).
- `native-service/src/cb_guest/entry.rs`: service uses `MediatorEval`.
- `native-service/src/registry/reg_ops/invoke.rs`: sets/clears cb-guest mediation context for
  the raw kernel callback path (host `root_eval` hooks remain gated out for cb-guest).
- `native-service/src/cb_guest/cap/wire.rs` and `program0/src/proxy/host/wire.rs` unchanged:
  P0↔P1 relay opcodes 1-13 are `OPEN_DISK..TRY_LOCK,RELEASE_LOCK`; nested/buffered eval are
  direct kernel imports in current ABI, not relay opcodes. No opcode-13 conflict in this wire.

## Host validation (independent target `/tmp/opencode/target-bonfires`)

- `cargo fmt --all -- --check`: PASS
- `rustc --test source_lints/mod.rs -o /tmp/opencode/skrynia-lint-tests-bonfires`: PASS (7/7)
- `cargo clippy -p skrynia-native-service --all-targets --locked -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity`: PASS
- `cargo test -p skrynia-native-service --locked`: PASS (2 unit + 35 integration)
- `cargo test -p skrynia-native-service --doc`: not executed (`rustdoc` missing)

## wasm32 toolchain / artifacts / runtime validation

- Integration kit documents reproducible wasm32 build-std procedure for 006a0d3 artifacts
  (`/workspace/skrynia-v3-integration-kit/wasm-build-record/README.md`, prebuilt 006a0d3 hashes
  `skrynia_program0.wasm=65396f61...`, `skrynia_native_service.wasm(cb-guest)=665d47dd...`).
  This session updates source to call `eval_buffered`; new artifact hashes and end-to-end
  runtime validation depend on rebuilding from current HEAD and running against a kernel+P0
  implementing buffered-eval + eval-delegate mediator capability (see below).
- **Not yet completed in this continuation:** rebuild current wasm32 P0/cb-guest artifacts with
  exact recorded hashes, and execute real POST+P0+P1+P2 adversarial HTTP tests on a pinned
  compatible kit/kernel (004c01b+ mediator-capable P0) proving authorized scoped P2 execution,
  invalid/sibling/root denial, trap/cancel/restart, and A-B-A CAS code-only rollback.

## Remaining blockers / coordination

1. **Pinned compatible runtime:** need kernel binary/commit with `eval_buffered` + mediated
   callback provider semantics (004c01b/8dWY lineage) and P0 that `eval_delegate`s forwarded
   service; pinned kit P0 `d166d974` uses plain `eval` (no mediator capability) so new cb-guest
   (imports eval/eval_buffered) is not compatible with it.
2. **Real compiled P1/P2 HTTP proofs:** pending artifact rebuild and execution on coordinated
   kit (owner H5) with P0 security/behavior coordination (wB1) and kernel specialist (hQGj,
   004c01b / 8dWYQhFw8BiBLUqpXSccAZ-enhance). No duplicate kernel caches.
3. **Board/Antonina:** cannot post to Antonina issue tracker from environment; record exact
   SHAs/hashes/tests for owner handoff.
4. **No production/secrets/Ivanka.**
