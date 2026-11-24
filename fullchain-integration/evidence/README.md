# Native WebPush full-chain evidence (PR79 evidence-fix)

`FULLCHAIN_OK_2bc4a20.log` is captured stdout from a real run of the
standalone `fullchain-integration` harness built from source commit
`2bc4a2040aa2571ed9ebbb43df3e1b3331b173ca` (stacked on PR79 head
`30bb38caf26f737e6330dddece82eaf1984a34c0`). The PR head after this
evidence commit adds only these documentation files, so rebuilding the
final head produces the same harness binary and the same output.

## Reproduce current source

The historical log above records its own commit and artifact digests. The
current Native components workflow builds the pinned generic kernel and
the current root proxy, native service and edge, generates the echo guest
from the compiled-service test fixture, and runs this harness against them.

For an equivalent local run, stage those five trusted build outputs under
the names `kernel`, `root.wasm`, `service.wasm`, `edge`, `echo.wasm`.
The trusted build must supply `SHA256SUMS` in that directory, with one
lowercase SHA256 digest, two spaces, and the artifact name per line.
Set FC_ARTIFACTS to the absolute directory and run
`cargo run --manifest-path fullchain-integration/Cargo.toml --locked`.
Expect `pin_verify=ok`, five SHA256 lines, scenario evidence and
`FULLCHAIN_OK`. The digest manifest protects against unintended artifact
substitution; it is not an authenticity proof for an untrusted producer.
Do not generate a fresh manifest from untrusted supplied binaries.

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
