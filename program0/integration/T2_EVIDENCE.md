# T2 exact-head evidence (2026-11-26)

Host `marceline-dev`. Repo `ottojung/skrynia-tools`, base T1 exact
`b5c9fe05e832913be6e919d4a171de5ef180de4b`, branch
`antonina/t2-program0-target-abi-dBzuOan7NbY6JDpNbcJjok-counter-20261126`.

## Commits

- `3ba4c454` feat(program0): target-ABI token verifier, CAS RAM allocator,
  generic eval and T3-coordinated `p0.invoke` relay.
- `d6558aa1` test(program0): disposable real-kernel fixture, protocol and
  blocker record.
- evidence commit: this file (head reported on the T2 board comment).

## Artifacts actually built

| Artifact | SHA-256 |
|---|---|
| `skrynia_program0.wasm` (release, wasm32-unknown-unknown) | `985d238b72a6554001f9642240f5d1a2d28fc1c23d9075fc3a61e988a7bd33bb` |
| echo test guest WASM | `50383e0b7e7e2cc27fbd758af851f9280ab74a11e0f6a046c7e99835df10259e` |
| trap test guest WASM | `3f899343ef54c1ad1609bb7d504e18a0740814a65c85fd1fac8bf1b13bf3674e` |
| kernel @ K4 `b8eff72` | `c82ab50285c1b61f45dc24ca2dcf4b33ee13a820601e6a7ec3eea92cd30da43b` |
| kernel @ K0 qualification `a96d143` | `96d4e12c780ae9bb009569c45af9709295e6536c959e3d4c02e5b93c4a677ca0` |

WASM import audit: 9 `skrynia` imports with exact target signatures
(`read1`, `write1`, `cas1`, `open_ram`, `truncate`, `seek`, `open_disk`,
`close`, `eval`); exports include `skrynia_main`, `memory`,
`__indirect_function_table` and `invoke`.

## Gates run (all green at final source head)

- `cargo fmt --all -- --check`: clean.
- `rustc --test source_lints/mod.rs` + run: 7/7 structural lint tests pass;
  build-time lints pass for all crates including `program0/`.
- `cargo test --locked --offline --all-targets`: all suites green
  (17 pre-existing lib tests + 6 new P0 tests + other crate suites).
- `cargo clippy --locked --offline --all-targets -- -D clippy::correctness
  -D clippy::suspicious -D clippy::perf -D clippy::complexity`: clean.
- `cargo build --locked --offline --release --target wasm32-unknown-unknown
  -p skrynia-program0`: real WASM artifact above.

## Disposable real-kernel validation

`python3 program0/integration/target_p0_http.py --kernel ... --p0 ... --guest
... --trap-guest ...` on a fresh 0700-style temp root with a fake bearer:

- kernel `b8eff729` K4 RAM fix: `ALL_OK=True` (echo, wrong-token denial,
  child-trap recovery, 16 concurrent per-request RAM-isolation echoes).
- kernel `a96d1435` latest K0 qualification branch (fresh-store eval):
  `ALL_OK=True` on the same artifact set.

Logs: `/tmp/opencode/t2-evidence/real-kernel-final2.log`,
`/tmp/opencode/t2-evidence/real-kernel-k5.log`; gate log
`/tmp/opencode/t2-evidence/gates-final.log`; workdirs
`/tmp/opencode/t2-target-p0-validation-final2` and `...-k5`.

## Not claimed / blockers

- Callback-dependent nested P2 guests remain BLOCKED (synchronous P1 suspension
  cannot service P2 callbacks); only byte queues and no-callback nested EVAL
  are relayed.
- Framed ops 5-10 return ENOTSUP; four-argument `copy` is owner-unresolved and
  deliberately absent.
- No production, no PR, no merge, no kernel edit, no old-ABI acceptance.
