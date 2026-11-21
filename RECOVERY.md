# Recovery state — Theorem Lab real-feedback P2 browser acceptance

Owner: `Jmf1xmEs6yST2sOvBNYUTN`. Created after the 2026-11-21 ~20:46 UTC container
restart that lost `/tmp/opencode` worktrees. This worktree is under persistent
`/workspace` and is based on the already-pushed PR54 commit `2e644e15...`.

## Durable facts (verified after restart)

* DRAFT skrynia-tools PR54: `https://github.com/ottojung/skrynia-tools/pull/54`
  head `2e644e15b8c5260643930597ba79d9c188afce47`, base `main`, open/draft.
  Do not recreate or amend; this branch is separate.
* Real P2 WASM persists at
  `/workspace/skrynia-v3-integration-kit/wasm-target/wasm32-unknown-unknown/release/theorem_lab_feedback_p2.wasm`
  SHA256 `5a26657858f515d29deaff012fc0109ea2d03a428d19895255239e8b780ad2bc`.
* Kernel `8da114ff...`, P1 `d3686527...`, echo P2 `dc28e490...` verified present.
* Clean dumb P0 PR53 source fetched at `fdf0d02` (worktree
  `/workspace/skrynia-recovery-theorem-p0-20261121`); rebuild needed (old
  `ec74523b...` binary was under lost `/tmp`).

## Work in progress

* `harness/` contains disposable copies of the apps PR305 (head
  `e6c9ff51ba678ab5ea4a8ef187207fc43b45cd90`) browser harness and feedback JS,
  fetched via the GitHub API for inspection. They are **not** production code.
* PR305's harness uses an **echo P2** and **gateway-mediated appdata CAS**; that is
  not acceptance for the real feedback P2. The replacement harness must drive the
  real compiled feedback P2 through P1's self-namespace callback, with the
  trusted edge acting only as a public-identifier/protocol adapter (no CAS).

No production, DNS, TLS, Ivanka, real credentials or user data are touched.
