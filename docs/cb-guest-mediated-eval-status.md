# cb-guest mediated nested-eval adaptation status

Branch: `agent/temperate-cb-guest-mediated-eval` (HEAD updated this continuation) on tools PR #48 lineage (`agent/persist-svc-qualify-bonfires@282ed2b`, includes `7ca12bb`, `282ed2b`).
Kernel ABI references: nested mediation `6bc52ec/7695fd3/ccbbbdf` (`docs/nested-eval-abi.md`), buffered bridge `004c01b` on `agent/hQGjVJwW8gkAfcPE3WnB1s/8dWY-buffered-eval` (`docs/buffered-eval-abi.md`, stacked atop nested reentrancy/PR117 lineage).

## Adapted (this branch)

- `program0/src/proxy/abi.rs`, `program0/src/proxy/entry.rs`: P0 forwards service via `skrynia.eval_delegate` (generic capability delegation) so forwarded P1 may act as immediate mediator with eval capability. P0 remains token check/strip + opaque forward only (no app/business/scoping semantics, no special Program1).
- `native-service/src/cb_guest/eval/{mod,cb,kernel}.rs`: mediator callback (`skrynia_mediator_cb`, indirect-table index) mediates P2 `invoke` requests through `appdata::cb::guest_callback` under app-scoped delegated rights (self invoke_id namespace only). `MediatorEval` calls kernel `skrynia.eval_buffered(code_ptr,code_len,req_ptr,req_len,res_ptr,res_cap,cb_idx)` (004c01b ABI) with immediate-parent callback mediation, returning buffered P2 output bytes for registry `Request::Invoke`.
- `native-service/src/cb_guest/entry.rs`: service uses `MediatorEval`.
- `native-service/src/registry/reg_ops/invoke.rs`: sets/clears cb-guest mediation context for raw kernel callback path (host `root_eval` hooks gated out for cb-guest).
- `native-service/src/cb_guest/fd.rs` (`aa6333a`): read/write `skrynia_main` input/output FDs via direct kernel stream imports `skrynia.read/write` (not P0 relay), since transferred guest FDs are inaccessible in provider callback context.
- `native-service/src/cb_guest/cap/wire.rs` and P0 reconciled relay wire unchanged in opcode numbering: P0↔P1 relay opcodes 1-13 are `OPEN_DISK..TRY_LOCK,RELEASE_LOCK`; nested/buffered eval are direct kernel imports, not relay opcodes.

## Host validation (independent target `/tmp/opencode/target-bonfires`, this continuation)

- `cargo fmt --all -- --check`: PASS
- `rustc --test source_lints/mod.rs ...`: PASS (7/7)
- `cargo clippy -p skrynia-native-service --all-targets --locked -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity`: PASS
- `cargo test -p skrynia-native-service --locked`: PASS (2 unit + 35 integration, 0 failed)
- `cargo test -p skrynia-native-service --doc`: **not run** (`rustdoc` missing in environment)

## Exact wasm artifacts

Procedure per `/workspace/skrynia-v3-integration-kit/wasm-build-record/README.md` (host `rustc 1.93.0`, `cargo 1.93.0`, `wasm-ld 14.0.6` `/gnu/store/2a6hs7mwkl9abl9bxmm4akq2akq9b9ip-lld-14.0.6/bin/wasm-ld`, sysroot snapshot `/tmp/opencode/rust-src-snap-1.93.0`, `RUSTC_BOOTSTRAP=1`, `-Z build-std=std,panic_abort`, `--target wasm32-unknown-unknown --release`, `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR=/workspace/skrynia-v3-integration-kit/wasm-target`):

- P0 (root proxy, PR51/aa6333a wire-reconcile + missdir + clamp, **no trace**): `skrynia_program0_pr51_f4a5270_wire_reconcile_aa6333a_missdir_clamp_4f1c4a4e.wasm`, **48664 bytes**, sha256 **`4f1c4a4eabc3a2e3475806efdd0901d815b340f1e939dcfba50f623ab59df64c`**. Source basis: `/tmp/opencode/skrynia-tools-p0-wire-reconcile-f4a5270 @ 5d4fe8d` dirty state with `host/{io,control,directory}.rs` clamp/missdir fixes; built outside tools PR49 (P0 owned by wB1) and copied to kit artifacts.
- P1 cb-guest (aa6333a, export-table): `skrynia_native_service_aa6333a.wasm`, **133129 bytes**, sha256 **`d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2`**. Imports: `skrynia.read`, `skrynia.write`, `skrynia.eval_buffered`, `skrynia.invoke`. Exports: `__indirect_function_table`, `skrynia_mediator_cb`, `skrynia_main`.

Superseded: 882fe384 (missing `__indirect_function_table`, breaks cb_idx>=0 eval_buffered callback); 6805f76e/febf3fdb/6e4f41e2 P0 variants (6805f76e/febf3fdb fail due to semantic -11 response overflow not retried by cb-guest call.rs; 6e4f41e2 has trace and hits kernel "compile root: limit").

## Real compiled HTTP E2E (no mocks) — 2026-11-21

Kernel: PR118 249ba342 (`/workspace/skrynia-v3-integration-kit/artifacts/skrynia-server_pr118_249ba342`, sha256 `1a2ad3e1528735301b8aa115c7988028dfa90a77751ec219a23c8e80a0cfb552`, 53554544 B). No kernel builds performed.

Fixed triad: kernel PR118 + P0 `4f1c4a4e...` + P1 `d3686527...`. P0 scope meta: `security/program/programs%2Froot.wasm` with ops=0xfff, prefix=`userland`. Token-gated HTTP POST.

**29/29 PASS** in `/tmp/opencode/e2e_cbguest.py`:

- Register echo P2: HTTP 200 OK, invoke_id matches.
- Lookup: HTTP 200 OK, code hash matches registered code.
- Invoke registered echo P2: HTTP 200 OK, body echoes `hello-p2-invoke`.
- ListPrograms/GetActive: HTTP 200 OK.
- Register appdata P2; invoke mediated appdata PUT/GET through P1 callback: HTTP 200 OK, inner protocol OK, GET returns value.
- Malicious sibling cross-namespace appdata PUT (P2 uses different invoke_id cap): denied (inner err 13).
- Operator isolation: register with wrong operator cap denied (err 9).
- CAS/ABA/restart: stage B, seal B, get active rev1, activate B (expected rev1) OK; stale activate (expected rev1 after activation) conflict err 10 with no current-hash leak in body form observed; restart; get_active/lookup show B; stage/seal A, activate rollback with current rev -> OK; lookup returns A.
- Trap P2: register OK, invoke returns err 4, server stays alive and serves subsequent lookup.

## Root cause / fix

Continuation 8 register/list/get-active/invoke/appdata err 4/5/7 was caused by P0 callback response handling: `host/io.rs` READ, `host/control.rs` RANDOM, `host/directory.rs` NEXT_ENTRY returned semantic wire OVERFLOW (-11) when requested length exceeded `dc-4`. cb-guest `call.rs` retries only on *transport* invoke -11 (buffer too small), not on semantic response code -11 inside a 4-byte reply. With cb-guest initial 256-byte invoke buffer and 8192-byte READ requests, this produced deterministic Disk(-11) -> err 7/4/5 depending on op default. Fix: clamp requested length to `min(requested, limit, dc-4)`. Missdir normalization (-5 -> -2 for missing open_directory) also included (from febf3fdb). **This is a P0 compatibility fix in the reconcile P0 source, not in tools PR49; coordinate with wB1/kit owner.**

## Remaining blockers / coordination

1. **P0 ownership/merge:** clamp+missdir P0 fix is in `/tmp/opencode/skrynia-tools-p0-wire-reconcile-f4a5270` dirty state (not committed/pushed here as tools PR49 code). wB1 should own/land the canonical P0 wire+clamp fix; 4f1c4a4e is an integration artifact for evidence.
2. **Kernel PR119 FS capability qualification:** not addressed here (no kernel changes/builds). Coordinate with hQGj.
3. **Broader qualification still not run:** full CI/independent security review on exact heads, fmt/lints/clippy/full make test on kernel heads, production/DNS/TLS/real creds/secrets/CLI/Ivanka. Doc-tests not run (`rustdoc` missing).
4. **Antonina posting:** environment has no Antonina posting/API credentials; evidence recorded in this doc and test output. Report to user for w4wg/ed2/8Tas posting.
