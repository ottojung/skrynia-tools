# Real-HTTP ordinary-service A→B→A CAS rollback + forged/unready negative

Owner: `agent-mkOdrtngd1HaLbcj6jDcQk-governess` (host `marceline-dev`).
Issue: `ed2MwvKbbtdHjMZi7ITTx5-dolokhov`. Base: `skrynia-tools` branch
`agent/persist-svc-callback-fixes-qW2Emr2jXayvHtelVsFoFP-voronezh` @ `ea4f152`
(commits `f280b21`, `d837600`, `0680a7e`, `ea4f152`).

This increment extends the prior echo-only real-HTTP E2E with a **code-only
A→B→A CAS rollback** (monotonic revisioned pointer, O(1) active record, all
historic releases retained) and **forged / unready-code negatives**, executed
through a real token-gated Program0 and generic kernel. No Program0 business
logic, no special kernel Program1 identity, no Node filesystem mock, no
production/DNS/TLS/real credentials/app migrations.

## What `rollback_e2e.py` drives

Self-contained Python stdlib harness; it builds two tiny WASM code guests in
memory (`skrynia_main` writes `CODE_A`/`CODE_B`) so the rollback is proved by
the INVOKE response changing with the active pointer hash, not by a mock read.

1. `REGISTER(A)` → 32-byte `invoke_id`, active `hash=SHA256(A)`, `revision=1`.
2. `STAGE(B)` → `hash=SHA256(B)`; re-stage of A returns the identical hash
   (immutable content-addressed blob).
3. **UNREADY negative:** `ACTIVATE(B)` before `SEAL(B)` must be rejected.
4. `SEAL(B)` then `ACTIVATE(expected_rev=1, B)` → `revision=2`, INVOKE→`CODE_B`.
5. `SEAL(A)` then `ACTIVATE(expected_rev=2, A)` → `revision=3`, INVOKE→`CODE_A`
   (code-only rollback restores the original release).
6. **STALE-CAS negative** (`expected_rev=1` again) and **FORGED-HASH negative**
   (never-staged hash) must be rejected.
7. `LIST_VERSIONS` shows both historical hashes; final `GET_ACTIVE` is
   `revision=3`, `hash=SHA256(A)`.

Exit 0 only if all 17 assertions pass; exit 1 on any failure.

## Pinned inputs and exact heads

- Kernel: `skrynia-server-exact-c39bcbf` (git `c39bcbf`), SHA256
  `f9a704a8f6f246b2c8fe4bee5efe1c207a63e4dc997a3b6fbf6959c69b1aa704`,
  at `/workspace/skrynia-independent-kernel-audit-20261121/bin/`.
- Tools source: `ea4f152ce225b9497aa7a9e808bdf7f985891d6e` (verified via
  `git ls-remote`). Workspace `Cargo.lock` SHA256
  `839205d0675df2d0a44680a3763ce4903d1141fe7aab1d8d5c1a36c9312bc556`.
- The originally compiled blobs quoted in the prior handoff (Program0
  `8a03330d…`, ordinary P1 `fc96db3d…`) were verified at run start, then
  deleted from `/tmp/opencode` by environment cleanup before the final run.
  They were **rebuilt from the exact source commit `ea4f152`** with the recipe
  below; rebuild hashes are recorded here so the result is reproducible.

### Reproducible rebuild recipe (durable dir `/workspace/mkOdrtngd1HaLbcj6jDcQk-governess-rollback-e2e-20261121`)

Reconciled rust-src sysroot `sysroot/` (copy of rustc 1.93.0 rust-src with
placeholder checksums stripped from `library/Cargo.lock`, `rustc` wrapper
rewriting `--print sysroot`); `wasm-ld` LLD 14.0.6 from the immutable Guix
store. From a clean `ea4f152` checkout:

```
WASMLD=/gnu/store/2a6hs7mwkl9abl9bxmm4akq2akq9b9ip-lld-14.0.6/bin/wasm-ld
RUSTC=$WS/sysroot/bin/rustc RUSTC_BOOTSTRAP=1 CARGO_BUILD_JOBS=2 \
RUSTFLAGS="--remap-path-prefix=$WS/sysroot=/__skrynia_sysroot__ \
 --remap-path-prefix=/home/lubko/.cargo/registry/src=/__cargo_registry_src__ \
 -C linker=$WASMLD -C link-arg=--export-table"
# Program0 (scope baked at compile time):
SKRYNIA_P0_FS_PREFIX=programs SKRYNIA_P0_FS_WRITE=1 \
CARGO_TARGET_DIR=$WS/target-p0 cargo build --locked -Z build-std=std,panic_abort \
 --target wasm32-unknown-unknown --release -p skrynia-program0
# Ordinary service (cb-guest):
CARGO_TARGET_DIR=$WS/target-p1 cargo build --locked -Z build-std=std,panic_abort \
 --target wasm32-unknown-unknown --release -p skrynia-native-service --features cb-guest
```

Rebuilt hashes:
- Program0 `dacbdf5b8d436e3d3266a435beaa63f16e272e42432678cf5f08d69750ce66a1`
- ordinary P1 (clean) `5b44a62fdc1d9c0f404d4550002f3c6027743aaae78f56e75b045ecd4c750f4b`

## Recorded results

| Run | Inputs | Result |
| --- | --- | --- |
| GREEN real HTTP | kernel c39bcbf + rebuilt P0 `dacbdf5b` + clean P1 `5b44a62f` | 17/17 PASS, exit 0 |
| RED real HTTP | same kernel/P0 + buggy P1 `1203cfcd` (readiness guard removed) | `UNREADY_NEGATIVE` st=0 (accepted), `FORGED_HASH_NEGATIVE` accepted, exit 1 |
| GREEN source | clean `ea4f152` + this commit | fmt clean; lints 7/7; tests 38+4+3 pass; clippy clean |
| RED source | guard removed (`red-injected-defect.patch`) | `unready_and_forged_hashes_cannot_be_activated` FAILED, RC=101, left 0 vs right 1 |

RED real-HTTP body for `UNREADY_NEGATIVE` decoded to an active-state record with
the unsealed hash B at `revision=2`, i.e. the service accepted unready code.
GREEN returned protocol status `1` (error code `8`) and left the pointer at A/1.

Skipped: doctests (rustdoc binary absent in this Guix env); full workspace clippy
(blocked by env `libgcc_s.so.1` for unrelated tools build script). No PR opened
via API (no `gh` token); branch pushed for manual draft PR.
