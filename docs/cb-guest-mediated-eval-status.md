# cb-guest mediated nested-eval adaptation status

Branch: `agent/temperate-cb-guest-mediated-eval` (based on PR #48 `agent/persist-svc-qualify-bonfires@282ed2b`).
Kernel ABI reference: `ccbbbdf1e771` / `6bc52ec` compose-eval (`docs/nested-eval-abi.md`, PR #116, base PR115).

## Adapted in this branch

- `program0/src/proxy/abi.rs`, `program0/src/proxy/entry.rs`: P0 forwards service via
  `skrynia.eval_delegate` (generic capability delegation) instead of plain `eval`, so the
  forwarded P1 may act as an immediate mediator with eval capability. P0 remains token
  check/strip + opaque forward only (no app/business/scoping semantics).
- `native-service/src/cb_guest/eval/{mod,cb,kernel}.rs`: added cb-guest mediator callback
  (`skrynia_mediator_cb`, indirect-table index) that mediates P2 `invoke` requests through
  `appdata::cb::guest_callback` under app-scoped delegated rights (self-namespace/invoke_id
  only), plus direct kernel `skrynia.eval` import and `MediatorEval` replacing `NoEval`.
- `native-service/src/cb_guest/entry.rs`: service uses `MediatorEval`.
- `native-service/src/registry/reg_ops/invoke.rs`: sets/clears cb-guest mediation context
  for the raw kernel callback path (host `root_eval` hooks remain gated out for cb-guest).
- `native-service/src/cb_guest/cap/wire.rs` and `program0/src/proxy/host/wire.rs` unchanged:
  P0↔P1 relay opcodes 1-13 are `OPEN_DISK..TRY_LOCK,RELEASE_LOCK`; nested eval is a direct
  kernel import in compose-eval ABI, not a relay opcode. No opcode-13 conflict in this wire.

## Host validation (independent target `/tmp/opencode/target-bonfires`)

- `cargo fmt --all -- --check`: PASS
- `rustc --test source_lints/mod.rs`: PASS (7/7)
- `cargo clippy -p skrynia-native-service --all-targets --locked -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity`: PASS
- `cargo test -p skrynia-native-service --locked`: PASS (2 unit + 35 integration)
- `cargo test -p skrynia-native-service --doc`: not executed (`rustdoc` missing in environment)

## Blockers (precise)

1. **wasm32 P1/P2 artifacts not built (no fake success):** `rustc 1.93.0` stable
   (`/gnu/store/r4i9jyz2mlvvyww6mwcgacmapm4rjllm-rust-1.93.0`), no `rustup`, no installed
   `wasm32-unknown-unknown` rustlib. rust-src 1.93 present in sysroot and
   `/workspace/skrynia-buildstd-sysroot/lib/rustlib/src/rust/library/...` (and Guix rust-src
   stores) but `RUSTC_BOOTSTRAP=1 cargo build -Z build-std=std,panic_abort --target wasm32-unknown-unknown`
   failed in this environment during std dependency resolution (`getopts`/`vex-sdk` checksum
   mismatch between lock files while fetching build-std deps, even with `--offline` attempts
   showing lock/registry state issues). No prebuilt wasm32 std rlibs found in `/gnu/store`.
2. **Buffered Invoke I/O bridge for fd-based nested eval:** compose-eval ABI transfers child
   I/O as kernel FDs `(in_fd,out_fd)` taken from mediator's fd table; cb-guest mediator has no
   guest capability to create buffer-backed stream FDs for registry `Request::Invoke` byte
   input/output. `MediatorEval::eval` therefore returns `-4` (permission/capability unavailable)
   rather than misrouting streams. Real P2 execution for buffered invoke requires coordinated
   generic ABI for guest buffer/pipe stream FDs or an explicit buffered-eval mediation contract.
3. **Real POST+P0+P1+P2 adversarial HTTP tests not run:** requires wasm32 P0/P1/(P2) artifacts
   and a pinned/integrated kernel runtime implementing ccbbbdf/6bc52ec mediation ABI; not
   available here and no kernel builds duplicated. Host Rust tests remain green but are not
   equivalent to real compiled wasm32/HTTP mediation proofs (scoped authorized success, sibling/
   root FD denial, trap/cancel/restart, CAS durability).
4. **Board/Antonina coordination:** cannot post to Antonina/PR issue tracker from this
   environment; requires owner/reviewer handoff.
