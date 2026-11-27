# T8 evidence — bounded nested staging and strict frames (2026-11-26)

Owner `agent-5qfgJHOL2ggIRwOK8T0D3a-anatole`, host `marceline-dev` (verified).
Base is the independently reviewed T2 head
`49a17d97fd8a3268dbdfc8ef70a07fbfc58fd8e9`; tested source head is
`a3e785959ae8986124370c467d644aa166bf5e6f` on branch
`antonina/t8-2KZTD0yrkW8sEbtwZdGVCn-governor-5qfgJHOL2ggIRwOK8T0D3a-20261126`.
Only `program0/` was edited: no kernel, T3/T6, app, production or main change.

## What changed

- Op 15 stages at most `SLOT_SIZE - SLOT_CODE_AT` bytes, checked before the
  first `write1`. A larger declared length returns `-6` with no RAM write,
  no eval and no neighbour-slot CAS corruption. The nested child context adopts
  the shared RAM cursor on success and trap, then its frame/reply/callback
  queues are dropped and the parent context is restored.
- `code::put` takes an absolute limit; path staging, both RENAME paths and
  random fill use their own caps. All descriptor handlers require full frames
  and reject trailing bytes before any kernel effect; missing OPEN_DISK/MKDIR
  mode is INVALID. READ/WRITE/NEXT_ENTRY reject oversized requests rather than
  truncating them. A non-EOF negative READ status is an error, not partial
  success. Strict Clippy findings in the overlapping P0 files were repaired.
- Program 0 remains a dumb token verifier and opaque byte relay: the same 17
  fixed `skrynia` imports, exported `invoke`/`skrynia_main`, no app policy, no
  new ABI, no CLI and no production action.

## Host gates (source head `a3e78595`)

| Check | Result | Log SHA-256 |
|---|---|---|
| `cargo +1.93.0 fmt --all -- --check` | rc=0 | `e3b0c442…` |
| `rustc --test source_lints/mod.rs` + run | 7/7, rc=0 | `f8d8f889…` |
| `cargo test --locked --offline --workspace --all-targets` | rc=0, 18/18 P0 tests | `53c85866…` |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` | rc=0 | `a28dcfd5…` |
| wasm32 release build `-p skrynia-program0` | rc=0 | `916f5276…` |

New P0 regressions: `oversized_op15_code_is_rejected_before_any_write`,
`malformed_op15_frames_never_eval_or_stage`, `nested_trap_leaves_the_parent_context_usable`,
`random_fill_is_capped_to_one_slot`, `rename_second_path_cannot_reach_the_code_area`,
`trailing_or_incomplete_descriptor_frames_have_no_effect`. The pre-existing
`denied_auth_never_leaks_a_busy_slot` sends 18 bad tokens (SLOTS+2), then a
valid token succeeds and all 16 CAS bytes are FREE.

Non-vacuity: the same `oversized_op15_code_is_rejected_before_any_write` test
added to a disposable worktree at base `49a17d97` FAILS (`left: 0`, `right: -6`),
i.e. the base stages the whole frame and evals. Log `35618a88…`; the
disposable worktree and target dir were removed afterwards.

WASM SHA-256 `657e6655e31e3065238c45b5e5fa9c7ce9af90ca229fb7a7f0bfebc27d1f7a0b`.
Section reader: exactly 17 `skrynia` imports and exports `memory`, `invoke`,
`skrynia_main`, `__indirect_function_table` (plus data markers); `invoke` type
`(i32,i32,i32)->i64`. The WASM build emits only the pre-existing `feed_cb`
dead-code warnings; it is not a strict-clippy gate.

## Disposable real-kernel validation

Kernel `kernel-k4-b8eff72` SHA
`c82ab50285c1b61f45dc24ca2dcf4b33ee13a820601e6a7ec3eea92cd30da43b`;
the exact P0 SHA above was pinned in every run. Fake bearer, temporary dirs,
no production token, DNS, TLS or subscriber data. Logs in
`/tmp/opencode/t8-governor-logs/`; harness copies and workdirs in
`/tmp/opencode/t8-governor-harness/`.

| Harness | Result | Log SHA-256 |
|---|---|---|
| `target_p0_http.py` (wrong-token, trap recovery, 16 concurrent isolated requests) | `ALL_OK=True`, rc=0 | `30e8a4d4…` |
| `target_p0_ops.py` (ops 5-10, descriptor guest `85ea9327…`) | `OPS5TO10_PASS`, rc=0 | `f7c0f448…` |
| T3/T6 published `nested_cb.py` bound only to new P0 path/SHA/workdir | `T3_T6_NESTED_CB_PASS` 5/5, `P1-OWNED-CB-ROUNDTRIP-OK`, rc=0 | `5ca050f6…` |
| `target_p0_ops.py` with new `oversize_guest.rs` (SHA `9b57c333…`, declared `u32::MAX` and cap+1 truncated frames) | `T8_FRAMES_PASS`, rc=0 | `abe93644…` |

The new guest's only import is `p0.invoke`; it writes its verdict through the
existing framed WRITE opcode. The harness was extended with optional `--want`
and `--label` arguments; defaults preserve the original OPS5TO10 invocation.

No merge, PR promotion, deployment, production or qualification claim is made;
the issue stays open for review. @ottojung
