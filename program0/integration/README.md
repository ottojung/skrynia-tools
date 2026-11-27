# Dumb Program0 HTTP integration fixture (PR53)

Executable, non-mock HTTP integration fixture for the clean dumb/opaque Program0.

- Real compiled kernel + P0 + aa6333a P1 over HTTP.
- Bounded disposable temp dirs; fake raw 32-byte test bearer; no real credentials/app data.
- No hardcoded private Guix/`/tmp/opencode` paths in source; all artifact paths are CLI args.
- CI is **not** a release/security gate for this fixture: existing GitHub CI jobs do not run this v3 stack.

## Verified artifacts (2026-11-21, 4389433 kernel)

| Artifact | SHA-256 |
|---|---|
| kernel `skrynia-server` @ 4389433 | `39a5ce174f76361f79023c00b6db456a92be43083d1b0cb5260d765a3feb1212` |
| dumb P0 `skrynia_program0.wasm` (PR53 6834cf5) | `ec74523be4d537969b6a9b93d702ba5f550a1205ca7fa85c3dc3261c5e1645dc` |
| P1 `skrynia_native_service_aa6333a.wasm` | kit artifact (pin/check as used) |

Kernel 4389433 FD callback rights tests (clean 4389433 worktree, not security-qualified): `fd_scope`/`mode_esc` 4 passed.

## Run

```bash
python3 program0/integration/dumb_p0_http.py \
  --kernel /path/to/skrynia-server \
  --p0 /path/to/skrynia_program0.wasm \
  --p1 /path/to/skrynia_native_service_aa6333a.wasm \
  --echo-guest /path/to/echo_guest.wasm \
  --kernel-sha256 39a5ce174f76361f79023c00b6db456a92be43083d1b0cb5260d765a3feb1212 \
  --p0-sha256 ec74523be4d537969b6a9b93d702ba5f550a1205ca7fa85c3dc3261c5e1645dc
```

Optional: `--workdir /tmp/opencode/dumb-p0-http` to keep logs in a bounded disposable dir, `--keep` to retain it.

## Cases covered

- Positive: register, lookup, P1->P2 invoke echo, appdata write/read (0xff User-root write mode), appdata CAS, restart persistence, lookup/get after restart.
- Negative: wrong operator cap denied, sibling appdata isolation, wrong bearer denied, malformed/missing bearer denied, shadow verifier in User dir denied.

## Caveats

- Do **not** claim security-qualified/release acceptance on 4389433 until hQ PR119 security patches land and are qualified.
- Run only against trusted/canonical artifacts; this fixture executes the supplied kernel binary.

## Bounded native-service large-read regression

The fixture additionally checks a 256,605-byte AppPut/AppGet round trip over
real HTTP, kernel, dumb Program0 and ordinary native P1, with fake data.
Each 8,192-byte file READ needs 8,196 bytes of provider callback capacity
(4-byte status plus data). A default 256-byte buffer clamps each read to
252 bytes and repeated callbacks can exhaust bounded Wasmtime fuel.

This optimizes the ordinary P1 guest, NOT kernel fuel or Program0 authority.
Pin the compiled kernel/P0/P1 artifact hashes and run the same fixture against
both old and new P1 binaries. Preserve the WASM indirect callback table export
(--export-table) when building P1; without it nested child invocation fails.

## Target-ABI P0 fixture (T2, 2026-11-26)

`target_p0_http.py` replaces the old-ABI fixture for the target port. It boots
a disposable compiled kernel at the K4 RAM fix (`b8eff72`), stages the new P0
WASM as root plus a minimal single-`invoke` child, and verifies byte echo,
wrong-token denial, child-trap recovery and 16 concurrent per-request RAM
isolation cases. No real token, no production, no kernel patch.

Build (rustup 1.93 toolchain, wasm32 std installed there):

```bash
cargo build --locked --release --target wasm32-unknown-unknown -p skrynia-program0
rustc --edition=2024 --target wasm32-unknown-unknown --crate-type=cdylib \
  -C opt-level=s -C panic=abort -C strip=symbols \
  program0/integration/guests/echo_guest.rs \
  -o program0/integration/guests-built/echo_guest.wasm
# same for trap_guest.rs
python3 program0/integration/target_p0_http.py \
  --kernel /path/to/skrynia-server --p0 /path/to/skrynia_program0.wasm \
  --guest program0/integration/guests-built/echo_guest.wasm \
  --trap-guest program0/integration/guests-built/trap_guest.wasm
```

The old `dumb_p0_http.py` exercises the retired bulk-callback ABI and is kept
only as historical evidence; it is not target acceptance. See
[`TARGET_ABI.md`](../TARGET_ABI.md) for the invoke protocol and blockers.
