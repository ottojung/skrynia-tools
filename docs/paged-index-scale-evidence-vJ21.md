# Bounded ready-version pagination — source and exact-stack evidence

Owner-authorized development increment for Antonina issue
`7X0twPgWYhSfBcCUBTZlwS-poor`. Host `marceline-dev`, 2026-11-24. No production,
deploy, TLS/DNS, kernel-limit or shared-kit change.

## Root cause (reproduced)

`native-service/src/registry/store_io/ready/listing.rs` called `Disk::list`
(OPEN_DIR + one NEXT_ENTRY per entry + CLOSE) and then `is_ready` (OPEN_DISK +
READ + EOF + CLOSE) for **every** ready marker before `page()` sliced the
result. With 197 markers the shared callback budget was exhausted
(independent `diag_list.mjs`: 197 OK / 198 native err 7 on this host; prior
agent recorded 198/199 on its build). Page limit could not help because all
markers were read first. `LIST_PROGRAMS` used the same all-directory scan.

## Fix

A new generic, immutable, page-addressable sorted index lives in
`native-service/src/registry/store_io/index/`:

- fixed-depth root -> directory -> leaf structure (256 / 128 / 124 entries),
  so a page touches one meta, one root, a bounded number of directories and
  at most a few leaves, independent of history size, hash distribution and
  cursor position;
- keyset cursors are the existing 32-byte hash — no wire or client change,
  arbitrary and sparse cursors resume at the containing leaf and never skip;
- copy-on-write nodes with a single atomic meta commit; a one-slot pending
  intent bridges the O(1) ready marker and the index so a crash or trap can
  never leave a sealed version unlistable or a false-ready listing;
- shared by ready versions and program IDs (register inserts, revoke
  tombstones); activation is untouched (fixed 42-byte pointer + direct marker).

No kernel/gateway/P0 change and no increase of any kernel limit.

## Validation

Rust (rustup 1.93.0, cached registry, `--offline`):

- `cargo fmt --all -- --check` clean.
- build-script structural lint clean (50 LOC / 10 entries / sibling imports).
- `cargo test -p skrynia-native-service -p skrynia-program0 --locked --all-targets --offline`:
  27 + 35 + 7 + 10 + 22 + 4 tests pass; 0 fail. Four new regression cases:
  bounded calls per page (<=32) with zero directory enumerations at 1000
  versions; adversarial 16-byte shared prefixes with sparse keys and
  arbitrary/stale cursors; pending-intent recovery and discard; program index
  listing 250 IDs with tombstoning.
- `cargo clippy ... -D clippy::correctness -D clippy::suspicious -D clippy::perf
  -D clippy::complexity` clean.

Exact compiled stack (release kernel `a1dc4119…`, P0 `44e845a4…`, trusted edge
`9fa4df14…`, echo `dc28e490…`, no rebuild of those). P1 rebuilt byte-reproducibly
from this source with rustup wasm32 + cached registry:

- baseline source `73d8fa1` reproduced kit P1 `c7239fc5…` byte-for-byte;
- fixed P1 `c0ac3357a67c3b09d15a2907ea08c8329569e9c0b034ca53cede661d51250820`.

Command:

```
cd <isolated checkout>
SKRYNIA_P1_OVERRIDE=<target-wasm>/skrynia_native_service.wasm \
SKRYNIA_P1_OVERRIDE_SHA256=c0ac3357a67c3b09d15a2907ea08c8329569e9c0b034ca53cede661d51250820 \
E2E_EVIDENCE_OUT=<evidence>/scale1000-fixed-evidence.json \
node tests/web-admin-v3/p1_admin_scale_1000.mjs
```

Result: **78/78 checks PASS** (`ALL_OK=true`), including `SCALE-LIST` (1001
hashes in 4 pages), `ADAPTER-scale`, `O1`, O2-O5 O(1) activation with a
byte-exact private-state diff, fail-closed denials and `RST5` (all 1001 ready
versions listable after kill/restart). Evidence
`/workspace/skrynia-poor-listfix-vJ21XVcGxwFqHAxP2YbdX1-20261124/evidence/scale1000-fixed-evidence.json`
sha256 `ecb695c849402129cbeeeb30420f27bb59029a54b89fe03cf0371842856921b1`;
log sha256 `47988abf75c676cf4e509fd3cc20b741cd0f48e6d290e6e0d79ffaf0fc1f2733`.

The shipped default pin remains the exact PR102 artifact; the harness override
is hash-gated and for source-change validation only. Release/cutover still
requires owner qualification of a rebuilt compiled kit.
