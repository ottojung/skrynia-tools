# Theorem Lab feedback — secure ordinary P2 guest

Standalone ordinary guest fixture demonstrating a least-authority P2 that stores
Theorem Lab feedback behind the native P1 (`skrynia-native-service`) self-namespace
callback. It is **not** kernel / Program0 / v2 code. The browser is an untrusted
caller: it knows only a 32-byte `invokeId` and can never obtain an external write
capability for the P2 namespace.

## Why this exists

`skrynia-apps/source/theorem-lab/feedback.js` used a retired public-write store
route (`GET/POST/PUT /_skrynia/store/theorem-lab/feedback-v1`). This guest replaces
that with a mediated write path:

* browser -> HTTP `OP_INVOKE(3)` with `invokeId` and a bounded JSON payload;
* P2 imports only `skrynia.invoke`, `skrynia.read`, `skrynia.write`;
* P2 reads/writes its own namespace through P1's `guest_callback`, which allows
  `AppGet` when `read_cap == invoke_id` and `AppPut` when `write_cap == invoke_id`
  using `WriteAuth::SelfNs(invoke_id)`;
* an external `AppPut` needs `write_cap w` with `SHA256(w) == namespace`, so knowing
  `invokeId` alone cannot forge a write into the P2 namespace.

## Contract (v1)

Input (JSON, <= 64 KiB):

```json
{"v":1,"op":1,"invokeId":"<64 lowercase hex chars = 32-byte invoke id>",
 "entry":{"id":"e1","comment":"Looks good","pageUrl":"https://example.test/theorem",
          "htmlChain":[]},
 "fragments":{},"updatedAt":"2026-11-21T00:00:00Z"}
```

* `invokeId` must be the exact 32-byte id the caller used in `OP_INVOKE` (hex encoded).
* `entry.id`: 1..128 chars, ASCII alphanumeric or `-_:.@` only.
* `entry.comment`: non-empty, <= 4096 bytes. `pageUrl` <= 2048 bytes.
* `htmlChain`: <= 16 entries, each `sha256:<64 hex>`.
* `fragments`: <= 16 entries, keys `sha256:<64 hex>`, values <= 16 KiB (opaque,
  never interpreted as markup by this guest).
* `updatedAt`: <= 64 chars.

Output (JSON): `{"ok":true,"v":1,"op":"submit","id":"e1","created":true,"retries":0}`
or `{"ok":false,"error":"bad_input|io|busy|<reject-reason>"}`.

State is a JSON document under key `theorem-lab/feedback-v1` in the P2's own
namespace: `{version, updatedAt, htmlFragments, feedback[]}`, bounded to 512 KiB
and 256 entries. Writes use SHA256 digest CAS with bounded retries; duplicate
`entry.id` is idempotent (`created:false`). Under sustained contention a submit
may return `{"ok":false,"error":"busy"}`; no partial or lost update is possible.

### Exact sample bytes for UI integration (deterministic test vector)

```
{"entry":{"comment":"Looks good","htmlChain":[],"id":"e1","pageUrl":"https://example.test/theorem"},"fragments":{},"invokeId":"1111111111111111111111111111111111111111111111111111111111111111","op":1,"updatedAt":"2026-11-21T00:00:00Z","v":1}
```

length 241 bytes; SHA256
`68c165304d4ba9298f3a9cd5285e73ddc111d69ccf17bc777aea5e2f8d134427`.
The `invokeId` above is the deterministic fixture value `0x11` repeated 32 times
(hex encoded); in deployment the UI must put the real 32-byte invoke id (hex) obtained
from the program registration/lookup, never an operator or admin capability.

## Build (canonical isolated recipe, no duplicate caches)

```
RUSTC=/tmp/opencode/rust-src-snap-1.93.0/bin/rustc \
RUSTC_BOOTSTRAP=1 \
CARGO_TARGET_DIR=/workspace/skrynia-v3-integration-kit/wasm-target \
CARGO_BUILD_JOBS=2 \
RUSTFLAGS="-C linker=/gnu/store/2a6hs7mwkl9abl9bxmm4akq2akq9b9ip-lld-14.0.6/bin/wasm-ld" \
cargo build -Z build-std=std,panic_abort --target wasm32-unknown-unknown --release
```

Verified artifact SHA256
`5a26657858f515d29deaff012fc0109ea2d03a428d19895255239e8b780ad2bc`; imports are
exactly `skrynia.invoke`, `skrynia.read`, `skrynia.write`; export is
`skrynia_main(input_fd, output_fd) -> i32`.

## Tests

```
python3 tests/nonvacuity.py                   # std-only assertion gate, runs in CI
cargo test --lib --tests                      # host contract/merge/CAS unit tests
python3 tests/real_kit_p2.py --kernel ... --p0 ... --p1 ... --p2 ...
```

The real-stack fixture is **not** a CI gate: this crate is standalone
(`[workspace]` in its Cargo.toml) and the pinned artifacts must be supplied by
an operator. It is documented as a manual/NOT-RUN-in-CI acceptance, not as
exact-head proof. `tests/nonvacuity.py` proves the two security assertions
demand denial/non-leak: it feeds the exact adversarial responses the old
assertions were blind to (a successful forge, and a leaked JSON state blob
containing `"id":"secret"`) and fails if either would pass. It is invoked by
the root-crate test `tests/p2_assertion_gate.rs`, which the existing
`cargo test --locked --all-targets` CI step runs; no workflow edit is needed.

Fixture pins are the canonical available kit artifacts: kernel `3508938…`
(the board-published D1/D2-fixed generic kernel; the earlier `8da114ff…`
pre-fix binary fails this fixture), P0 `d57ab27a…` (the lost `ec74523b…`
artifact is replaced by the PR53-source rebuild), P1 `83a65a2f…` (the retired
aa6333a `d3686527…` lineage is replaced by the clean-P1 kit artifact), and
this P2 `5a266578…`. All four SHA-256s are verified from bytes at startup.

The Python fixture runs the real compiled kernel + P0 + P1 + this P2 over HTTP
with fake 32-byte bearers in disposable dirs. It verifies registration, submit,
idempotency, concurrent CAS (no lost update; structured busy on contention),
write-cap forging denial, sibling namespace isolation, unauth invoke denial,
and restart persistence.

## Coordination note (UI engineer 48mQ via Antonina / issue `ygD` / `w4wg`)

No direct messaging tool was available in this environment. This file and the
draft PR are the contract hand-off artifact. The obsolete public-write header in
`skrynia-apps/source/theorem-lab/feedback.js` must be removed by the UI owner;
feedback must be submitted through `OP_INVOKE` with the sample shape above. All
HTML-derived content is represented only as bounded `sha256:` references and
opaque fragments, so no script-bearing markup crosses the guest boundary.
