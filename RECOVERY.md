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

## Result (actual feedback P2, not echo)

`harness/real_feedback_p2_chromium.py` drives the unmodified apps PR305
`feedback.js` in real headless Chromium through nginx HTTPS -> trusted edge
(protocol adapter ONLY, no appdata CAS) -> kernel -> clean dumb P0 -> ordinary
P1 -> THE feedback P2.

Final run log: `real-p2-chromium-run4.log` SHA256
`cae8c34420759456a3dcc271e245e7f43536b19714a78ae8fc6f8f6a4653ed2f`,
**22 PASS / 0 FAIL, ALL_OK=True**, retained work dir
`/workspace/tl-real-p2-a7d3eofm`.

Verified: real P2 registration + direct invoke; browser Save; persistence via
real public `OP_APP_GET`; invoke-read path; genuine concurrent real-P2 CAS
conflict mapped to one 409, browser retry succeeded; concurrent no lost update;
kernel restart persistence; deny bad Host/Origin/session/non-POST/wrong kernel
token/sibling namespace; no admin cap in page and no P0 token in logs.

P0 note: rebuild from the same PR53 source commit `fdf0d02` produced a different
WASM SHA (`5dd3e382...` locally) than both the earlier `ec74523b...` and the
pinned `d57ab27a...`; byte-identical reproducibility is NOT claimed. The pinned
independently-validated `d57ab27a...` artifact was used for the run above.

## Superseded run / final architecture-correct result

The earlier `real-p2-chromium-run4.log` (`cae8c344...`) used an app-specific
translating edge and is **not** acceptance. It was superseded after owner review
(apps PR305 comments 7069149069 / 7069158605).

Final: `harness/real_feedback_p2_chromium.py` now uses a **generic framing-only
edge** (Host/Origin/session + private P0 bearer framing only; no Theorem JSON
parsing/rewriting, no gateway CAS). The browser fix lives in its own skrynia-apps
draft PR306 (base = PR305 branch; PR305 itself unchanged). Browser sends the real
P2 v1 input verbatim (numeric `op:1`, public `invokeId`); read is the scoped
public `OP_APP_GET`.

Final log `real-p2-chromium-generic-edge-final.log` SHA256
`cfaf5882826fd735a1b5eacf450ef38935e3afa3fa8742b3743f0abe76b87996`:
**ALL_OK=True, 22 PASS / 0 FAIL**. Includes an executable raw-byte equivalence
check: every browser OP_INVOKE payload hash is observed unchanged at the kernel
(only the 33-byte bearer prefix added), a genuine bounded CAS retry, concurrent
no lost update, restart persistence and all auth negatives.

No production, DNS, TLS, Ivanka, real credentials or user data are touched.

## Blocker
No Antonina board (`ygD`/`w4wg`) tool or MCP resource is available in this
environment; board reporting was done via GitHub comments on apps PR305 only.
