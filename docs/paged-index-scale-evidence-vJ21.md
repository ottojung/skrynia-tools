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
  27 + 43 + 7 + 10 + 22 + 4 tests pass; 0 fail. Four new regression cases:
  bounded calls per page (<=32) with zero directory enumerations at 1000
  versions; adversarial 16-byte shared prefixes with sparse keys and
  arbitrary/stale cursors; pending-intent recovery and discard; program index
  listing 250 IDs with tombstoning; zero key survives, missing committed root fails closed, 2499 removals keep page reads <=32, 100 idempotent inserts keep file count <=16.
- `cargo clippy ... -D clippy::correctness -D clippy::suspicious -D clippy::perf
  -D clippy::complexity` clean.

Exact compiled stack (release kernel `a1dc4119…`, P0 `44e845a4…`, trusted edge
`9fa4df14…`, echo `dc28e490…`, no rebuild of those). P1 rebuilt byte-reproducibly
from this source with rustup wasm32 + cached registry:

- baseline source `73d8fa1` reproduced kit P1 `c7239fc5…` byte-for-byte;
- fixed P1 `b0e94de5bd2c43256b4dd28fad3829e1f991b86128fc53f33b0ff064e1970a6b`.

Command:

```
cd <isolated checkout>
SKRYNIA_P1_OVERRIDE=<target-wasm>/skrynia_native_service.wasm \
SKRYNIA_P1_OVERRIDE_SHA256=b0e94de5bd2c43256b4dd28fad3829e1f991b86128fc53f33b0ff064e1970a6b \
E2E_EVIDENCE_OUT=<evidence>/scale1000-final64-evidence.json \
node tests/web-admin-v3/p1_admin_scale_1000.mjs
```

Result: **78/78 checks PASS** (`ALL_OK=true`), including `SCALE-LIST` (1001
hashes in 4 pages), `ADAPTER-scale`, `O1`, O2-O5 O(1) activation with a
byte-exact private-state diff, fail-closed denials and `RST5` (all 1001 ready
versions listable after kill/restart). Evidence
`/workspace/skrynia-poor-listfix-vJ21XVcGxwFqHAxP2YbdX1-20261124/evidence/scale1000-final64-evidence.json`
sha256 `af16590d1a0167bf4d43b7faf102a8e4cb9114805bc1d212ca4dd174262704e2`;
log sha256 `ca1f5a6b830c133c71a2b3d8ab8e7a93171a417d9099cf2226eb795b4b9b3e55`.

The shipped default pin remains the exact PR102 artifact; the harness override
is hash-gated and for source-change validation only. Release/cutover still
requires owner qualification of a rebuilt compiled kit.

## Review-driven corrections and legacy compatibility

Addressing the exact-head NO-GO review: query-time replay and page reads now
hold the mutation lock (busy maps to err 11 with no effects); leaf occupancy is
an explicit flag so the all-zero key is never a sentinel and an absent cursor
is distinct from a cursor equal to zero; program removal physically deletes
the entry (no unbounded tombstone scan); node decoding validates length,
checksum, ordering and occupancy and a missing committed node is an error;
u32 counter overflow is rejected before any marker commit; a bounded retired
ledger lists exactly the fresh/retired node ids so normal operation reclaims
COW nodes (file count stayed fixed at 2147 in the 1000-version run).

Storage compatibility is explicit: pre-index ready/active state has no index
meta and therefore fails closed with a native error; there is no silent empty
listing. Such state requires an explicit owner-controlled migration (replay
STAGE/SEAL/register under the new build), which is outside this source change.
The source-only `native-admin` CI job no longer auto-runs the artifact-backed
scale harness; the pin remains unchanged.

Final journal hardening: the transaction journal now names target_generation,
fresh leaf/dir head+tail ids and old root/dir/leaf ids; the new root id equals
the target generation; empty head nodes are encoded as 0. Recovery accepts only
actual generation target or target-1 (0 only for target 1), otherwise fails
closed without cleanup, deletes the correct fresh set on uncommitted and old
set on committed, tolerates absent paths via a pre-read, and propagates real
delete errors so the journal is cleared only after successful reclamation.

## Explicit MAX_PAGE=64 (released-protocol decision)

The native P1 keyset page limit is now exactly 64: `limit=0` and `limit>64`
are rejected with err 1 (no silent truncation), and the shipped
`native_capabilities.js` adapter, strict client tests and E2E harnesses use
`PAGE_LIMIT=64`. This is an unreleased protocol, so no compatibility shim is
kept. Worst-case traversal is 64 singleton leaves plus 64 singleton
directories plus first-cursor skipped paths, within the observed callback
budget; an independent real-compiled-P1 sparse64 fixture (256 one-key
directories, boundary cursors) passed 11/11. The 1000-version harness now
walks 16 pages and remains 78/78 with the rebuilt P1. The scale harness no
longer skips absent artifacts: a missing pin is a hard failure. The
source-only `native-admin` job does not invoke the scale harness; the
compiled-stack job executes it with runner-built SHA-verified artifacts.
