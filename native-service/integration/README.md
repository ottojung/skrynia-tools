# Ordinary nonzero native-service P1 — clean current-main consolidation

This branch is based on clean dumb-P0 PR53 (`fdf0d02`, itself current tools
main `17b8b41` + 2 commits). It selectively imports **only** the ordinary
nonzero native-service P1 Rust crate from PR49 (`01e8a84`). It does **not**
merge or rebase PR49's 56-commit legacy stack, so the current-main trusted
edge gateway, web-admin and no-CLI scope are untouched.

Architecture boundary (unchanged):
- Program0 is a token/opaque proxy only. It has no app/registry/ CAS logic.
- Ordinary nonzero P1 owns grants, appdata, registry and CAS.
- P2 is an arbitrary user module invoked by P1.

## Build (real wasm32, shared kit target, jobs <= 2)

See kit `wasm-build-record/README.md`. The P1 requires the indirect function
table exported so the kernel can resolve the mediator callback;
`.cargo/config.toml` supplies `-C link-arg=--export-table` for wasm32.

    RUSTC=/workspace/skrynia-buildstd-sysroot/bin/rustc \
    RUSTC_BOOTSTRAP=1 CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR=/workspace/skrynia-v3-integration-kit/wasm-target \
    CARGO_ENCODED_RUSTFLAGS=... linker + export-table ... \
    cargo build -Z build-std=std,panic_abort --target wasm32-unknown-unknown \
      --release -p skrynia-native-service --features cb-guest --locked

P1 artifact from this branch source: `skrynia_native_service_p1clean_83a65a2f.wasm`
sha256 `83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a`.

## Pinned real HTTP acceptance

`run_acceptance.sh` runs PR53's `program0/integration/dumb_p0_http.py`
fixture (real compiled kernel + P0 + P1 + echo P2 over HTTP, no mocks) with
sha256-pinned inputs:

- kernel: secure PR122 bridge `8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f`
- P0: clean PR53 dumb proxy `ec74523be4d537969b6a9b93d702ba5f550a1205ca7fa85c3dc3261c5e1645dc`
- P1: this branch `83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a`
- P2 echo: `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5`

Covered: register + lookup + P1-mediated P2 invoke, AppPut/AppGet, CAS,
wrong-operator denial, sibling appdata isolation, restart durability,
wrong/missing/shadow bearer denial.

No production, DNS/TLS, real credentials, user private data or Ivanka changes.
