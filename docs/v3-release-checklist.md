# Skrynia v3 parity, migration and release checklist (draft)

Status: **draft working document** for milestone `dgKBALkpO7M7oKBHrZUzCT-shortly`, issue `2ntGmTZ0qiM98ky9Q7vA43-running`.
Scope: black-box differential parity gates, safe v2→v3 data migration/rollback validation, and release readiness evidence. This document does not authorize production mutation.

## Hard constraints

- **No production data mutation** by tests, harnesses, or migration rehearsal unless an explicitly reviewed runbook step authorizes it on a disposable copy.
- **No CLI in skrynia-tools**. Administration and deploy remain web/scripted per project rules; migration validation must not introduce operator CLI commands.
- **v3 kernel boundary**: one public kernel endpoint `POST /` with `Content-Type: application/octet-stream`, request body passed unmodified to Program 0. Legacy HTTP routes/methods/status/header semantics are provided only by the skrynia-tools compatibility HTTP facade; parity is measured at the facade wire boundary, not claimed for the kernel alone.
- **Pinned v2 baseline**: every differential claim must record the exact v2 server source commit/build artifact, skrynia-tools commit, configuration, and data snapshot provenance.
  - **Reference v2 server source (pinned for this increment, 2026-11-18):** Skrynia `https://github.com/ottojung/Skrynia.git` commit `10cbf0d5ad00224e1375a4f2482965f8d3513559` (`origin/main`, `v1.0.0-369-g10cbf0d` at clone time). Built locally as `cargo build --locked --package skrynia-server` (debug profile, rustc 1.93.0/cargo 1.93.0) producing `rust-server/target/debug/skrynia-server` in an isolated clone. Deployment-specific binary/config parity beyond this reference build remains to be confirmed against the authoritative deployed environment if different.
  - **Previous candidate note:** `c7b527e` observed in `/workspace/skrynia-rust-rewrite` was not used as this run's executable baseline; superseded by the directly built `10cbf0d` evidence above.
- **Prospective v3+facade target**: record Skrynia kernel commit, Program 0 WASM artifact/hash, skrynia-tools facade commit, and startup configuration (`SKRYNIA_ROOT_WASM`, data dir, limits).

## Differential parity gate (black-box)

Run `tests/http/differential_parity.py` (or equivalent orchestration) against an isolated disposable v2 instance and an isolated disposable v3+facade instance. Compare wire-level status, headers, and bodies per inventory case; record normalizations explicitly (e.g. date/order-insensitive JSON fields only when protocol allows).

### Coverage matrix

| Area | Required cases | Evidence |
| --- | --- | --- |
| Public/meta | `/health`, `/version`, `/push/vapid` status/body shape, CORS/preflight as applicable | |
| Object reads | GET/HEAD/Range/If-None-Match/If-Range/ETag/Cache-Control/Content-Type/404/416 | |
| Object mutations | POST create, PUT replace, DELETE, X-Skrynia-If-SHA256 CAS, 201/200/404/409/412/400 invalid precondition, path conflicts, forbidden precondition headers | |
| Listing | `GET /store?locations=...` auth, scope, pagination/`next`, sorting, empty/invalid matchers | |
| Grants/auth | POST /grants, GET grant, issue user, revoke user/grant, issuer vs user capability separation, narrowing/quota metadata, post-revocation denial | |
| Quotas | bytes/objects/grants/tokens/subscription accounting, exhaustion errors, concurrent pressure where observable | |
| Web Push | subscribe/update/delete, capability auth, immutable locations/events, delivery/ack/wake-up semantics as observable from black-box harness | |
| Concurrency | parallel conflicting creates/CAS/revocations/activations; no torn authoritative state | |
| Crash/restart | restart during/after commit points; recovery preserves committed state and rejects ambiguous replay assumptions | |
| Security regression | traversal/confused deputy/credential leakage in errors/logs/responses, unauthorized access, malformed payloads | |
| HTTP/proxy/browser | TLS reverse proxy forwarding, browser-compatible CORS/cache behavior, stable app origins unaffected by kernel POST-only design | |

### Performance baseline (warm simple writes)

- Measure warm simple write latency p50/p95/p99 on pinned v2 and candidate v3+facade in comparable isolated environments.
- Report methodology: payload sizes, concurrency, warmup, sample count, storage medium, CPU/memory limits, and whether durability/sync is included.
- Compare against Rust baseline; regressions must be explained and accepted before release.

## Safe v2→v3 data migration and rollback validation

Migration is a separate explicit validated export/import process. No automatic in-place production conversion.

### Export/import rehearsal (disposable copy only)

1. **Snapshot source**: take a filesystem-consistent copy of pinned v2 data directory/state (or approved export artifact). Record source commit, schema/version markers, object/grant/push state counts, and checksums.
2. **Export**: produce a deterministic export manifest (objects, metadata, grants/issuers/users as representable, subscriptions, active pointers/markers, protocol-visible state). Include format version and integrity hashes.
3. **Import into candidate v3+Program 0**: load into isolated v3 data directory with configured Program 0/facade. Verify import is idempotent or explicitly single-use as designed.
4. **Verification**: compare exported inventory vs imported readable state (counts, keys, bytes/digests, grant scopes/quotas/revocations, subscription records, pointer values). Run differential read/write probes on imported data where safe.
5. **Rollback validation**: demonstrate that original v2 snapshot remains intact and can be restarted as authoritative service; demonstrate v3 candidate can be discarded without affecting v2 snapshot. Document rollback trigger conditions and operator steps.

### Rollback triggers (examples)

- Differential parity failure on release-critical cases.
- Data verification mismatch, unknown mutation outcomes, or credential/quota inconsistency.
- Unacceptable latency/regression or crash/recovery defect.

## Release checklist

- [ ] VM spec 01 finalized with resolved ABI/open items and executable conformance evidence.
- [ ] Kernel 02 passes isolation, limits, fuzz/error, crash/confine, and benchmark evidence.
- [ ] Program 0 03 passes registry/set/invoke/auth/concurrency/restart tests.
- [ ] Compatibility facade 04 passes full differential inventory (no CLI added).
- [ ] Differential parity harness results published for pinned v2 vs v3+facade, including concurrency/crash/push/security/quota.
- [ ] Warm write latency p50/p95/p99 compared to Rust baseline and published.
- [ ] Migration export/import verification and rollback rehearsal completed on disposable data with recorded evidence.
- [ ] `make test`/lint/CI green in both repos as applicable; independent review completed.
- [ ] Reproducible deploy and rollback instructions published.
- [ ] Production cutover plan reviewed; old v2 data preserved; v2 Rust business logic removed only after green gate.

## Evidence log

| Date | Component | Commit/artifact | Command | Result | Notes |
| --- | --- | --- | --- | --- | --- |
| 2026-11-18 | v2 compatibility inventory coordination | skrynia-tools `9ce10b9` (+ PR #10 artifacts from `feat/v2-compat-inventory-harness-uTJ5794`) | `./tests/http/v2_compat_harness.sh check` | pass | `inventory_ok endpoints=17 corpus=12`; no facade implementation duplicated. |
| 2026-11-18 | pinned v2 baseline build (disposable) | Skrynia `10cbf0d5ad00224e1375a4f2482965f8d3513559` | `cargo build --locked --package skrynia-server` | pass | Debug binary built in isolated clone; rustc/cargo 1.93.0. No production data touched. |
| 2026-11-18 | differential harness expansion + generated plan | skrynia-tools `7c004fb`, `faca38c`, `afc601d` | `python3 -m py_compile tests/http/differential_parity.py`; `python3 tests/http/differential_parity.py --self-test`; `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --generate-plan` | pass | `self_test_ok generated_cases=43`; plan covers public/meta, conditional/range, store CRUD/auth negatives, listing, grants, quota, push validation categories driven by `skrynia-tools.v2-compat-inventory`. |
| 2026-11-18 | baseline self-consistency (disposable pinned v2 vs same build, fresh temp data dirs) | Skrynia `10cbf0d` + skrynia-tools `afc601d` | `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --baseline-self-consistency --v2-binary <.../skrynia-server>` | pass | 5 scenarios, 50 observations, 0 failures: `public_meta`(3), `store_reads_writes_conditionals_ranges_auth`(28), `grants`(9), `quota`(4), `push_validation`(6). Normalized instance-generated values (VAPID key, grant/user IDs). |
| 2026-11-18 | migration/rollback rehearsal preparation (disposable) | skrynia-tools `63a260d` | `python3 tests/http/migration_rehearsal.py plan`; disposable live-data manifest/snapshot/verify dry-run | pass | Snapshot+manifest+verify on disposable v2 data copy: `summary={"grant_files":1,"object_bytes":28,"objects":2,"push_files":2}`, manifests matched (`73a9045...`). No production mutation. |
| 2026-11-18 | local non-Rust validation | skrynia-tools branch | `cargo fmt --all -- --check`; `rustc --test source_lints/mod.rs -o /tmp/skrynia-lint-tests-jcsZ && ...`; `./tests/http/v2_compat_harness.sh check` | pass | Source lints 7/7. Full `cargo test --locked --all-targets`/clippy and CI/review not run in this increment. |

## Current status (2026-11-18, agent-jcsZAxSeTC0x4SsKXOKohF-screwed)

**Completed/checkable increments**
- Continued branch `feat/differential-parity-orchestration-kLZ4FhKF4i4gVv6CmECiJt` (PR #11 context) without duplicating compatibility facade implementation.
- Independently pinned and built reference v2 baseline server from Skrynia `10cbf0d5ad00224e1375a4f2482965f8d3513559` (debug) for disposable testing.
- Expanded differential harness to generate 43 inventory-driven cases and execute baseline self-consistency scenarios covering conditional/range/negative status, auth, grant issuance/revocation semantics, quota exhaustion, and push validation paths on isolated disposable v2 instances.
- Prepared migration/rollback rehearsal tooling and plan; validated snapshot/manifest/verify flow on a disposable data copy (no production mutation).

**Cannot run / not yet proven (precise blockers)**
- **v2-vs-v3(+skrynia-tools facade) differential:** no complete/integrated v3 kernel + Program 0 + compatibility facade candidate available on this host/branch context; `SKRYNIA_V3_FACADE_BASE_URL` candidate runs not executed.
- **Full inventory parity execution:** only implemented/runnable subset executed as v2 baseline self-consistency; cases requiring external HTTPS push delivery/ack, reverse-proxy/browser compatibility, concurrency/race stress, crash/restart/fault injection, security fuzzing, and exhaustive negative matrix across every environment-dependent code path remain.
- **Warm write latency p50/p95/p99 comparison vs Rust baseline:** not measured (needs agreed methodology and comparable v3 candidate environment).
- **Actual v3 import/export rehearsal:** import into v3 and post-import parity cannot run until v3 storage/protocol/facade exist; only disposable v2 snapshot/manifest/rollback-copy verification performed.
- **Full repo validation/CI/review gates:** skrynia-tools full `cargo test --locked --all-targets` and clippy not run this increment; Skrynia repo tests/CI not re-run beyond baseline build context; independent review not completed.
- **Production release/removal gates:** no reproducible deploy/cutover evidence against qualified environment, no rollback drill on deployment topology, and v2 Rust business logic must not be removed before all green gates.

**Reproducible commands (local evidence)**
- Harness checks: `python3 -m py_compile tests/http/differential_parity.py && python3 tests/http/differential_parity.py --self-test && python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --generate-plan`
- Baseline self-consistency (disposable): `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --baseline-self-consistency --v2-binary <path-to-pinned-skrynia-server>`
- Migration rehearsal dry-run helpers: `python3 tests/http/migration_rehearsal.py plan|snapshot|manifest|verify` on disposable copies only.

