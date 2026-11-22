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
- P0: clean PR53 dumb proxy rebuilt from PR53 source in this recovered environment `d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa` (the earlier `ec74523b...` artifact lived under `/tmp` and was lost in the container restart; exact byte reproduction was not possible here because the isolated sysroot snapshot was also lost, but this PR53-source rebuild passes the same acceptance unchanged).
- P1: sized file-read callback rebuild from the `a1cca7b` source (this branch) `6fb58853c908a9265c7ec0425cba2d75e74d0f34b3c9ee822d5be104f47b3699`. The earlier `83a65a2f...` pin predates the `a1cca7b` read-callback sizing fix and fails the 256,605-byte `appdata-large-get-exact` check; do not reuse it for acceptance.
- P2 echo: `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5`

Rebuild the P1 from its pinned source with the indirect function table export
preserved (`--export-table`), using the reproducible build-std procedure in
`wasm-build-record/README.md` of the integration kit:

    RUSTC=/workspace/skrynia-buildstd-sysroot/bin/rustc RUSTC_BOOTSTRAP=1 \
    CARGO_BUILD_JOBS=2 CARGO_ENCODED_RUSTFLAGS=$'-Clinker=<wasm-ld>\x1f-Clink-arg=--export-table' \
    cargo build -Z build-std=std,panic_abort --target wasm32-unknown-unknown \
      --release -p skrynia-native-service --features cb-guest --locked

Covered: register + lookup + P1-mediated P2 invoke, AppPut/AppGet, 256 KB
large-object byte-exact read, CAS, wrong-operator denial, sibling appdata
isolation, restart durability, wrong/missing/shadow bearer denial.

No production, DNS/TLS, real credentials, user private data or Ivanka changes.
