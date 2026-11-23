# Native WebPush full-chain evidence (PR79 evidence-fix)

`FULLCHAIN_OK_2bc4a20.log` is captured stdout from a real run of the
standalone `fullchain-integration` harness built from source commit
`2bc4a2040aa2571ed9ebbb43df3e1b3331b173ca` (stacked on PR79 head
`30bb38caf26f737e6330dddece82eaf1984a34c0`). The PR head after this
evidence commit adds only these documentation files, so rebuilding the
final head produces the same harness binary and the same output.

## Reproduce

1. Obtain the five pinned artifacts whose SHA256 values are printed in the
   log (they are gitignored build outputs, never committed) and place them
   in `fullchain-integration/../artifacts/` under the exact names shown.
2. `cd fullchain-integration && cargo build --locked`
3. `FC_ARTIFACTS=../artifacts ./target/debug/fullchain`
4. Compare stdout to the log: `pin_verify=ok`, five runtime-computed
   `SHA256 <name> <hex>` lines, the `EVIDENCE ...` lines and `FULLCHAIN_OK`.

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
