# Coordinator source/pin review — jNl6lEzTOrl6g8vSuryQRB (2026-11-26)

Review only. No production, deploy, service mutation, merge, PR promotion or
push to `main`; no Intent Record authored from a task prompt.

## Exact heads reviewed

- tools `antonina/target-full-tools-integration-20261126` @
  `37eea06ba7520ac4ea5d9f17e09e28f0d99a84c3`
- apps A3 head `a4a729b4f6093f27f77cef04c7ce9d03e5c092cb`
- kernel K10 head `db08cbc7496fafedc6a2e32df80395911b930879` (local only)

## Guest source identity (pin freshness)

`git diff --stat 3a4330a 37eea06 -- examples/theorem_lab_feedback_p2
native-service/src program0/src` shows **only**
`examples/theorem_lab_feedback_p2/tests/real_kit_p2.py` changed (12+/11-).
The P0, P1 and A3 P2 guest source files consumed by the published pins are
byte-identical, so these pins are source-fresh, not stale:

- P0 `3f2d40321c2d2af8a77cbc210f863c963492f1b021885a88d963e8b5bd957357`
- P1 `ec84e056a209d4d58d990a231070c1b55f21f7b564a676426fdcb3599d94fc23`
- P2 `19a592588d9f0987ec93b55837778aadf9673c6b35e814cacfa2f1d043e8d11a`
- payload `68c165304d4ba9298f3a9cd5285e73ddc111d69ccf17bc777aea5e2f8d134427`

The committed kernel pin `0339ca954b40a5907936add3edde5e146e625599008899dec6f6c816f6d2a7f0`
is a genuine K10 build but its exact bytes are no longer present on this host.
Fresh `db08cbc` rebuilds legitimately differ by embedded build path
(`3da1852e…`, `89fd1209…`, `9753d9fe…`); the stale pre-K10 `4ab7f3fc…` and
A3-recovery artifacts (`214dece7…` P2, `e480f928…` P0, `a7d20be0…` P1) are not
acceptable substitutes. A single canonical kernel digest and an explicit,
fail-closed pin-override path for fresh rebuilds remain open integration work.

## Deployment / admin ABI contract

- `web-admin-v3` emits native opcodes 9–14 only; `AppPut`/`AppGet` (6/7) are
  never emitted, no CLI/v2 `/store` facade exists, tokens stay server-side.
  Its real-stack receipt is pinned to older source heads and is historical,
  not exact-head acceptance.
- `publish-provider` and the publication-root WASM maintain disjoint roots,
  path grammar rejection and kernel-managed `flock` activation; the
  mailbox/provider direct POST is a diagnostic, not a deploy path.
- No HTTP deployment action was introduced while integrating A1/A2/A3.

The coordinated apps evidence with exact commands, live callback results and
the remaining migration list is
`apps/evidence/jNl6lEzTOrl6g8vSuryQRB-integration-20261126.md`.
