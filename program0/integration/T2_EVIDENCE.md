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
- At the original base only: framed ops 5-10 return ENOTSUP (the continuation
  section below implements them). Four-argument `copy` is owner-unresolved and
  deliberately absent.
- No production, no PR, no merge, no kernel edit, no old-ABI acceptance.

## Continuation — generic descriptor ops 5-10 and regression repairs (2026-11-26)

Owner `agent-FDGNmUxmAssq8qRwvny1wC-observed`, host marceline-dev (hostname
verified). Base is the exact T2 head `22c5db2e0524a9a28e244770c9c7db69fd054158`
on branch `antonina/t2-vQUJZ6fZ-ops5to10-FDGNmUxm-20261126`; only `program0/`
was edited. The earlier evidence above remains historical.

### What changed

- Framed ops `5` MKDIR, `6` DELETE, `7` RENAME, `8` OPEN_DIRECTORY,
  `9` NEXT_ENTRY and `10` SYNC are implemented on the six already-existing
  fixed K0 imports (`mkdir`, `delete`, `rename`, `open_directory`,
  `next_entry`, `sync`). No invented import, no kernel edit and no app policy.
  OPEN_DISK now relays its framed `u32` mode instead of hardcoding read-only;
  the kernel still enforces root capabilities.
- Root cause of the reported E_IO read-path failure: the invoke context's RAM
  cursor was initialised to zero while the real RAM fd sat just past the staged
  guest, so path bytes were staged at the wrong offset and the kernel opened
  stale bytes. The cursor is now inherited from staging and resynchronised
  after nested EVAL. The path area was widened so real 150-byte appdata paths
  are not rejected.
- 17:42Z regressions: auth now runs inside the outcome closure so denied or
  malformed tokens release the CAS slot on every result path; guest staging is
  capped at `SLOT_SIZE - SLOT_CODE_AT` and closes the source on every path;
  `alloc::ensure` probes capacity and only grows, never shrinking RAM outside
  the exclusive arena.
- Frozen `p0.invoke(ctx:i32, op:i32, value:i32)->i64`, import pair `p0`/`invoke`,
  selector 0 and `_start` child entry are unchanged. No blocking P1-suspending
  CB relay was added.

### Artifacts actually built

| Artifact | SHA-256 |
|---|---|
| `skrynia_program0.wasm` (release, wasm32-unknown-unknown, final source) | `bb3b8afb6bc6999015a9dc667b00733504ae8f703d6634436819a415c0f46c1b` |
| descriptor ops test guest | `85ea9327c9b3dbd6f299df61d508a0391e32d278f1722084853af104b3372270` |
| echo / trap guests (unchanged from T2) | `50383e0b…` / `3f899343…` |
| kernel @ K4 `b8eff729` | `c82ab50285c1b61f45dc24ca2dcf4b33ee13a820601e6a7ec3eea92cd30da43b` |
| T3/T6 P1 bound from head `d0497b7466626252f7a7fd9e09cff828e4b82241` | `e9a65426caa5902716c6a07ded6071829980ce4ee4ceab0d42ae6362fca4fcfc` |

WASM import audit: exactly 17 `skrynia` fixed imports (the original 11 plus
`mkdir`/`delete`/`rename`/`open_directory`/`next_entry`/`sync`), exports
`skrynia_main`, `memory`, `invoke`, `__indirect_function_table` and data symbols.

### Gates run at final source (all green)

- `cargo fmt --all -- --check` rc=0, log sha256
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `rustc --test source_lints/mod.rs` + run: 7/7 rc=0, log sha256
  `3e160d529425eeee9b6450894b6d7a5d322ede86421e6d22c0ab8a8c2740b314`.
- `cargo test --locked --offline --workspace --all-targets` rc=0, all suites
  green including 12 P0 tests (three new descriptor tests, three new
  regression tests); log sha256
  `c25f7b89fefc26fa9482acbfcfb394b40596008df17edb56a5a5443ef9735f57`.
- strict clippy rc=0, log sha256
  `ded36599a38c2bfbbb952313a5191c986bd8433e400b898fd1362e447af80bc0`.
- real wasm32 release build rc=0, log sha256
  `812f53e53a49e3cc532d14ee107903d4a638d9718e54e1c166ff948509b2674f`.

### Disposable real-kernel validation

All runs used the exact P0 SHA above, the K4 kernel, fake bearers and isolated
temp dirs; no production, no PR and no kernel/T3 edits.

- `OPS5TO10_PASS` — `target_p0_ops.py` stages the descriptor guest and drives
  MKDIR/OPEN_DISK(write+create)/WRITE/SYNC/RENAME/DELETE/OPEN_DIRECTORY/
  NEXT_ENTRY/SYNC through `p0.invoke`; body `OPS5TO10_OK`. Log sha256
  `f7c0f448a8b554c82bcc9bdc7f5aa9d7289f295bb07232be1e1e080b9e5ffd60`.
- `ALL_OK=True` — unchanged `target_p0_http.py` echo, wrong-token denial,
  child-trap recovery and 16 concurrent RAM-isolation cases. Log sha256
  `30e8a4d45194100104fe41ea7b0b60430f20fbf48c68fe94519593e1971992ad`.
- `T3_T6_NESTED_CB_PASS` — the published `nested_cb.py` from T3/T6 head
  `d0497b7` was bound verbatim except for the P0 path/SHA and workdir
  constants, using its exact P1 `e9a65426…` and four WASM guests. Nested echo,
  P1-owned appdata CB_GET/CB_PUT roundtrip, sticky failure, malformed import
  and wrong-bearer denial all pass. The outer roundtrip body is
  `01 00 18 00 00 00 50 31 2d 4f 57 4e 45 44 2d 43 42 2d 52 4f 55 4e 44 54 52 49 50 2d 4f 4b`;
  the inner body is `P1-OWNED-CB-ROUNDTRIP-OK`. Log sha256
  `5ca050f655f8eb911313232517635caa1261f99c58ab266173e0016e3aaaee47`.
  The previous exact-harness baseline with P0 `985d238b…` was
  `T3_T6_NESTED_CB_FAIL` (disk open E_IO in Registry::invoke staging).

### Claim boundary

No merge, deployment, production or qualification claim. P0 remains a token
verifier and generic descriptor proxy; it does not interpret app payloads,
does not relay callbacks into a suspended P1 and adds no kernel ABI. d81f16b
remains NO-GO for RAM resize. The issue stays open.
