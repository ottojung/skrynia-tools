# `boundedVersionEnumeration`: exact 1000-version test and capacity evidence

Development increment for Antonina issue `ZZYan9Ny0XiLvSBD1iIwCF-hall`.
Host `marceline-dev`; isolated clone and build dir; git SSH only. Disposable
in-memory disks and loopback processes only: no production data, credentials,
DNS/TLS, deploy or cutover. This document does **not** mark the release gate
verified — all twelve `docs/native-v3-release-gates.json` statuses remain
`pending` in the apps repository and the gate stays operator-held.

## What was added

`native-service/tests/cases/behavior/releases/scale1000/` adds eight
deterministic Rust tests against the generic ordinary-P1 registry on tools
main. They stage and seal **1000 distinct nonzero code versions** plus the
base registration (1001 ready markers) and then assert:

| Test | Property |
| --- | --- |
| `enumeration` | `list_ready` equals the exact sorted 1001-hash set; the native `ListVersions` opcode walks the whole history in 4 bounded pages of 256 with no gap or duplicate. |
| `cursor` | Malformed (31-byte) cursor is rejected with err 1; a sparse cursor resumes at the exact successor; a terminal cursor returns an empty page. |
| `activation` | CAS activation reads at most the 42-byte active pointer and one direct ready marker, performs zero history enumerations, and writes at most 42 bytes, with a full 1001-version history resident. Arbitrary earlier/later A-B-A across versions 1→2→3→4, stale revision denied. |
| `lifecycle` | Revoke (with exact present precondition) and re-register keep activation single-pointer and nonce-bound across revisions 3→4→5. |
| `recovery` | Restart (drop and reopen the registry on the same private disk) preserves the active record and full enumeration; an unsealed stage stays unactivatable; a removed final ready marker fails closed until explicit replay re-seal. |
| `capacity` | Wire page ceiling is exactly 256 (`limit=257` err 1, `limit=256` OK); the shipped directory-scan listing performs one enumeration and at least one read per ready marker (unbounded work). |
| `legacy` | An unsupported ready-marker format (version byte 2) makes listing fail closed with `BadRecord` and activation to that hash fail; the explicit owner-controlled migration — remove the unsupported marker, replay `stage_release` + `seal_release` — restores a listable, activatable version. |

These are **source-level** tests: they run the host-side P1 service logic with
injected `MemDisk`/`Metered` capabilities and the real wire codecs. They are
executable, non-vacuous assertions, not mocks of the behaviour under test, but
they are **not** compiled-WASM end-to-end coverage of the 1000-version listing.

## Real compiled-WASM corroboration (pinned, disposable)

Two harness runs used byte-verified artifacts from `/workspace/skrynia-exact-kit-73d8fa1`
(kernel-release `a1dc4119…`, dumb P0 `44e845a4…`, ordinary P1 `c7239fc5…`) plus
the pinned trusted edge gateway `9fa4df14…` and echo guest `dc28e490…`:

- `tests/web-admin-v3/real_native_e2e/p1_admin_ops.mjs`: **42/42 PASS**,
  `result_count=42`, `ALL_OK=true` — real native opcodes 9/10/11/12/13 and 15 through gateway +
  kernel + P0 + P1, small-scale list pagination, A→B→A revision CAS,
  stale/ABA/unsealed/forged/corrupt denials, restart-safe state.
  Evidence: `p1-admin-ops-pinned.json`.
- `tests/web-admin-v3/real_native_e2e/p1_capacity_ceiling_probe.mjs`: real
  compiled-P1 capacity ceiling — **last successful `ListVersions` at 198
  ready markers; first failure at 199 with native error 7 (IO/callback budget
  exhaustion)**. Evidence: `capacity-probe.log`.

## Honest capacity conclusion

The exact 1000-version enumeration is **not** achievable through the shipped
pinned P1 artifact (`c7239fc5…`): its full-scan `list_ready` reads every ready
marker before slicing a page, so the shared syscall callback budget is
exhausted at 199 markers. Page limits cannot help. The exact complete ordered
1001-hash enumeration is therefore proven **only at source level** by the Rust
tests above. Closing that gap on the compiled stack requires the separate,
unmerged paged-index build and an explicitly qualified rebuilt P1; that is not
part of this increment and is not a release or merge action.

## Unsupported-index migration outcome

On tools main there is no on-disk version index; readiness is a direct marker
directory. The exercised fail-closed path is an unsupported marker format:
listing returns `BadRecord` rather than a silent empty or partially trusted
list, and activation to that hash is denied. Migration is only the explicit
operator replay of `stage_release`/`seal_release` after removing the
unsupported marker — verified executable by the `legacy` test. There is no
silent/auto migration path and none is claimed.

## Validation run (exact head `98eaf1d8…`)

- `cargo fmt --all -- --check` — clean.
- Build-time structural lint (50 code LOC/file, ≤10 entries/dir, sibling
  imports) — clean; `rustc --test source_lints/mod.rs` self-tests 7/7.
- `cargo test -p skrynia-native-service --locked --all-targets --offline` —
  **119 passed, 0 failed** (8 new scale tests among them).
- `cargo test --locked --all-targets --workspace --offline` — **151 passed,
  0 failed**.
- `cargo clippy --locked --all-targets --workspace --offline -- -D
  clippy::correctness -D clippy::suspicious -D clippy::perf -D
  clippy::complexity` — clean (needed `LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu`
  on this host for build scripts).

## What remains for the gate

1. Build and byte-pin a paged-index P1 and re-run the full 1000-version
   enumeration through the real compiled stack (the source-level result is not
   a substitute).
2. Independent review of the exact branch head.
3. Coordinator/operator decision on any merge; gate status change and release
   remain operator-held. No production action is authorised by this evidence.
