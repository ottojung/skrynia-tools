# Paged-index ordinary P1: byte-pinned artifact and compiled-stack enumeration

Owner-authorized development increment for Antonina issue
`ZZYan9Ny0XiLvSBD1iIwCF-hall`. Host `marceline-dev`, 2026-11-25. This closes
the compiled gap left by the source-level 1000-version tests: the shipped
`c7239fc5` P1 full-scans every ready marker and fails at 199 markers with
native err 7 on the real compiled WASM stack.

## Source and lineage

Branch `antonina/paged-index-p1-bytepin-z21R3VPW-20261125`, based on tools main
`c3b3545bd7c1518ead2c78aff2183b2109095f70` (tree
`ee062d04352d4c9ace0f49fb074e53746f798b8c`), which already carries the
paged-index implementation in `native-service/src/registry/store_io/index/`
and the resolved wire contract (MAX_PAGE=64, native err 1 BADREQ for
overlimit/malformed pages, exclusive 32-byte SHA-256(code) raw-byte tail keys,
empty terminal page).

## Rebuild and byte pin

The P1 was rebuilt from that exact source in an isolated directory, offline,
with the rustc 1.93.0 build-std sysroot snapshot and wasm-ld 21.1.8:

```
RUSTC=/workspace/skrynia-buildstd-sysroot/bin/rustc \
RUSTC_BOOTSTRAP=1 CARGO_BUILD_JOBS=2 \
CARGO_TARGET_DIR=<isolated>/wasm-target \
RUSTFLAGS="-C linker=<lld-21.1.8>/bin/wasm-ld -C link-arg=--export-table" \
cargo build --locked --offline -Z build-std=std,panic_abort \
  --target wasm32-unknown-unknown --release \
  -p skrynia-native-service --features cb-guest
```

The archived artifact is `skrynia_native_service.paged-index.<sha256>.wasm`,
byte-attested in `SHA256SUMS`. Its digest is the pin consumed by the
compiled-stack harness and verifier:

- **P1 pin**: `c46e92f274b93cbbdc0a65bbd9a645a7ab24b21e3b450a340764d74ef5efa44a`

Note: lld 14.0.6 aborts with SIGFPE when linking this exact source (two
consecutive runs); wasm-ld 21.1.8 links cleanly. The resulting bytes are a
valid, distinct pin — byte-reproducibility of this specific artifact is
established by re-running the same recorded command, not by identity with the
earlier PR107 prebuilt `b0e94de5`.

## Compiled stack (all pins verified from bytes at startup)

- release kernel `a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1`
- dumb P0 `44e845a4ca1bfcf6713078ad40d2ce7d387595b08351a5a882bbc38549780c95`
- paged-index P1 `c46e92f274b93cbbdc0a65bbd9a645a7ab24b21e3b450a340764d74ef5efa44a` (this artifact)
- echo guest `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5`
- trusted-edge gateway `9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f`

All traffic is disposable loopback POST `/_skrynia/` through the shipped
binary adapter. No production data, credentials, DNS/TLS, deploy or cutover.

## Evidence

- `scale1000-compiled-enumeration-evidence.json` (sha256
  `038c8dab350074c433a9d508c47dbbecbdd39257ab41dc287137e69b12b3501c`) —
  `p1_admin_scale_1000.mjs` with the new P1 override: 78/78 checks,
  `ALL_OK=true`, all five pins byte-verified, `SCALE-LIST` 1001 hashes in 16
  pages of limit 64, `ADAPTER-scale` 1001, O(1) activation (76-byte request,
  42-byte record, exactly one changed file, 2149 files unchanged), A-B-A/CAS,
  revocation and kill/restart durability.
- `compiled-enumeration-verifier-evidence.json` (sha256
  `f80000fb1ee197ea997036c59e3e17146dabce285a194eda5b17905bfc77b0b9`) —
  `verify_compiled_enumeration.mjs`: 19/19 checks, `ALL_OK=true`, page sizes
  `[64 x 15, 41]`, all hashes SHA256
  `1bdce1d4d1c64a319aab47ff45327b385000c092aa77b9b5486fdd420e206c20`,
  strictly ascending raw-byte order, exact set equality with the STAGE
  hashes (no gap/duplicate), exclusive tail cursors equal to each previous
  page's last key, empty terminal page, and native err 1 on malformed
  31/33-byte cursors and limit 0/65.
- `capacity-ceiling-nonvacuity.json` (sha256
  `f61a381f41b1f2c1b41c954fce58d17f56a84609b72aa93ba236e1342f0d41e0`) —
  `p1_capacity_ceiling_probe.mjs` with the **unmodified** `c7239fc5` pin:
  `lastListOkAtReadyCount=198`, `firstListFailureAtReadyCount=199`,
  `failureNativeErr=7`. The old full-scan pin still fails.
- `p1-repro-sha256.txt` — independent fresh-target rebuild produced the
  identical P1 digest `c46e92f2…`.

Exact commands, counts and the baseline 8 scale1000 tests (119/0 native-service,
152/0 workspace at exact head `2a59099`) are in the board evidence comment and
the terminal handoff. This is development qualification only; no gate status is
claimed and the release remains operator-gated.
