# Native WebPush full-chain evidence (historical, NOT exact-head)

`FULLCHAIN_OK_2bc4a20.log` is captured stdout from an earlier real run of the
standalone `fullchain-integration` harness built from source commit
`2bc4a2040aa2571ed9ebbb43df3e1b3331b173ca` (stacked on PR79 head
`30bb38caf26f737e6330dddece82eaf1984a34c0`). It is **historical evidence**:
its pins (kernel `4c6d34f9…`, P1 `cf1322c3…`) are superseded by the current
pinned table `src/chain/pins/table.rs` (kernel `ac301e18…`, P1 `e9e5e1f3…`),
and it lacks the wake/ordinary verdicts the current harness always emits.
**Rebuilding the current head does NOT produce the same harness binary or the
same output**; that earlier claim was false and is withdrawn. No exact-head
fullchain run was performed for this evidence set (NOT RUN here). The
exact-head gate is the CI `compiled-stack` job.

## Reproduce

1. Obtain the five current pinned artifacts and SHA-256 values from
   `fullchain-integration/src/chain/pins/table.rs` (gitignored build outputs,
   never committed) and place them in `fullchain-integration/../artifacts/`
   under the exact names shown there. The historical log's pins differ.
2. `cd fullchain-integration && cargo build --locked`
3. `FC_ARTIFACTS=../artifacts ./target/debug/fullchain`
4. Compare stdout to the current pin table and the `EVIDENCE ...` verdict
   lines, including the wake/ordinary verdicts. Do not expect byte equality
   with the historical `FULLCHAIN_OK_2bc4a20.log`.

## Fail-closed (RED) check

Replace any one artifact with different bytes and rerun. The harness exits
non-zero at the `run/mod.rs` pin check, printing the expected and observed
SHA256 values; no kernel or edge process is spawned. Unit tests
(`cargo test --locked`) cover correct-pin accept, wrong-pin reject,
tampered-bytes reject and runtime-computed manifest (4 passed, 0 failed).

## Scope

Disposable loopback only. No real VAPID credential, push service, Chromium
notification event, DNS/TLS, production branch or subscriber data. Whole
release remains NO-GO pending the operator-held Chromium/real-provider,
five-app migration and independent exact-head review gates.
