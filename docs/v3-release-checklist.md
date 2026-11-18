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

## Current status (2026-11-18, continuation agent-V2n5T3VthTvOegCHwhYz26-consulted on branch `da85974`, PR #11 context)

**Completed/checkable increments (this continuation)**
- Continued parity branch commit `da85974` (`feat/differential-parity-orchestration-kLZ4FhKF4i4gVv6CmECiJt`, PR #11 context) without duplicating compatibility facade implementation. Extended `tests/http/differential_parity.py` with executable modes for: inventory-driven baseline self-consistency (existing), v2-vs-candidate scenario differential when an externally managed disposable candidate and credentials are provided (`--differential-scenarios`), concurrency/race checks (`--concurrency`), crash/restart durability checks (`--crash-restart`), security regression probes (`--security`), and warm write latency measurement (`--latency-writes`, p50/p95/p99 with recorded methodology). Extended `tests/http/migration_rehearsal.py` with executable disposable `live-rehearsal` (populate v2, stop, snapshot, manifest/verify copies, restart original and verify reads, confirm rollback copy untouched; v3 import step explicitly not run).
- Ran executable validation on isolated disposable instances using pinned v2 baseline binary `Skrynia 10cbf0d5ad00224e1375a4f2482965f8d3513559` (`/tmp/opencode/skrynia-parity-jcsZAxSeTC0x4SsKXOKohF-screwed-2ntGmTZ0qiM98ky9Q7vA43/skrynia/target/debug/skrynia-server`, debug, rustc/cargo 1.93.0, host marceline-dev). **No production data mutated.**

**Evidence (this continuation, all disposable/local)**
- Harness/inventory/lint checks: `python3 -m py_compile tests/http/differential_parity.py tests/http/migration_rehearsal.py` OK; `python3 tests/http/differential_parity.py --self-test` => `self_test_ok generated_cases=43`; `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --generate-plan` OK (43 cases); `./tests/http/v2_compat_harness.sh check` => `inventory_ok endpoints=17 corpus=12`; `cargo fmt --all -- --check` OK; `rustc --test source_lints/mod.rs -o /tmp/skrynia-lint-tests-parity-jcsZ && /tmp/skrynia-lint-tests-parity-jcsZ` => 7 passed.
- Baseline self-consistency (disposable v2 vs same pinned build): `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --baseline-self-consistency --v2-binary <v2bin> [--json-out ...]` => **PASS** (5 scenarios, 50 observations, 0 failures): `public_meta`(3), `store_reads_writes_conditionals_ranges_auth`(28), `grants`(9), `quota`(4), `push_validation`(6).
- Concurrency/race (disposable v2): `python3 tests/http/differential_parity.py --concurrency --v2-binary <v2bin> [--json-out ...]` => **PASS** (`ok:true`, workers=8, rounds=3): conflicting concurrent POST creates 1x201/7x409 per round, concurrent CAS replace 1x200/7x412, concurrent user issue/revoke consistent, post-stress health 200.
- Crash/restart (disposable v2): `python3 tests/http/differential_parity.py --crash-restart --v2-binary <v2bin> [--json-out ...]` => **PASS** (`ok:true`): acknowledged write durable/readable after kill+restart, continued writes after restart, 20/20 acknowledged burst writes readable after kill+restart, post-crash health 200.
- Security regression probes (disposable v2): `python3 tests/http/differential_parity.py --security --v2-binary <v2bin>` => **PASS** (`ok:true`): traversal/encoded traversal denied (400) without leak, missing/bad/empty/wrong-scheme auth denied (401), malformed grant bodies rejected (400), issuer cannot write store / user cannot issue users / out-of-scope write denied (403), auth error does not echo bearer token.
- Warm write latency p50/p95/p99 (disposable v2 baseline only): `python3 tests/http/differential_parity.py --latency-writes --v2-binary <v2bin> --latency-warmup 20 --latency-count 200 --latency-payload-bytes 128 [--json-out ...]` => **PASS** (`ok:true`). Methodology: POST create-if-absent, unique keys, payload 128 bytes, warmup 20, count 200, concurrency 1, localhost HTTP, disposable local filesystem data dir, includes request/response overhead, server defaults. Results (ms): p50=11.015, p95=20.853, p99=21.201, min=10.385, max=22.912, mean=13.198. **No v3+facade candidate comparison executed.**
- Migration/rollback rehearsal (disposable v2, no v3 import): `python3 tests/http/migration_rehearsal.py live-rehearsal --v2-binary <v2bin> [--evidence-dir ...]` => **PASS** (`ok:true`, `production_mutation:false`): populated representative state (3 objects incl. nested key, 1 child grant), filesystem-consistent snapshot/manifests matched (`source==rollback==import-candidate`, sha256 `f6d6b5...`), restart original and verified reads OK, rollback copy remained untouched; `v3_import_and_post_import_parity` explicitly `not_run` with reason (no integrated candidate). Artifacts written to evidence dir when requested (source/rollback/import manifests + result JSON). `python3 tests/http/migration_rehearsal.py plan` updated to version 2 documenting executable-now vs cannot-run-now.

**Cannot run / not yet proven (precise blockers)**
- **v2-vs-v3(+skrynia-tools facade) executable differential:** no complete/integrated v3 kernel + Program 0 + compatibility facade candidate available/runnable on this host in current context. Harness supports `--candidate`/`--differential-scenarios` and candidate credential args, but **not executed** (no candidate base URL/artifact). Observed components in `/tmp/opencode`: kernel checkout `skrynia-kernel-bpD4et6TjYTBjt4y1SgKs5-space-aoSNEmkx` (branch `agent/bpD4et6TjYTBjt4y1SgKs5-space/aoSNEmkx-kernel-v3-abi340`, built `target/debug/skrynia-server`), program0 checkout `skrynia-program0-2XLBbn6W9aDrwZyjfmSQ6n-eloquent` (wasm32 release build dirs present, no obvious `.wasm` artifact identified), facade scaffold in `skrynia-tools-uTJ5794-9VGwsIEvHNiNDouDYD6onF-july` (`src/wire/v2compat` handles only GET /health,/version,/push/vapid + preflight; no full store/grants/push/auth implementation) - insufficient for end-to-end parity run.
- **Warm write latency p50/p95/p99 comparison vs Rust baseline:** v2 baseline measured on disposable debug build; candidate v3+facade not available, so comparison not executed.
- **Actual v3 import/export and post-import parity:** blocked on missing v3 storage/protocol/facade/import procedure; only v2-side disposable snapshot/manifest/rollback-copy-untouched/restart verification executed.
- **Full exhaustive parity/security matrix:** executed subsets include strong black-box coverage (conditional/range/mutations/auth/grants/quota/push-validation scenarios, concurrency races, crash/restart, traversal/malformed-auth/issuer-user separation/scope/leak probes). Not executed: real HTTPS push delivery/ack/end-to-end subscription lifecycle against accepting endpoint, reverse-proxy/browser compatibility, TLS-specific behavior, external fuzzing campaigns, environment-specific deployment checks, and full differential against candidate.
- **Full repo validation/CI/review gates:** skrynia-tools local checks above passed; `cargo test --locked --all-targets` and `cargo clippy --locked --all-targets -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity` not run in this continuation. Skrynia repo tests/CI not re-run here. Independent review not completed.
- **Release/removal gates:** no reproducible deploy/cutover evidence against qualified environment, no production-appropriate rollback drill, v2 Rust business logic must not be removed before all green gates.

**Reproducible commands (this continuation)**
- Harness checks: `python3 -m py_compile tests/http/differential_parity.py tests/http/migration_rehearsal.py && python3 tests/http/differential_parity.py --self-test && python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --generate-plan && ./tests/http/v2_compat_harness.sh check`
- Baseline self-consistency (disposable): `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --baseline-self-consistency --v2-binary <path-to-pinned-skrynia-server>`
- Concurrency: `python3 tests/http/differential_parity.py --concurrency --v2-binary <path-to-pinned-skrynia-server>`
- Crash/restart: `python3 tests/http/differential_parity.py --crash-restart --v2-binary <path-to-pinned-skrynia-server>`
- Security probes: `python3 tests/http/differential_parity.py --security --v2-binary <path-to-pinned-skrynia-server>`
- Warm write latency (v2 baseline): `python3 tests/http/differential_parity.py --latency-writes --v2-binary <path-to-pinned-skrynia-server> --latency-warmup 20 --latency-count 200 --latency-payload-bytes 128`
- Migration/rollback rehearsal (disposable v2, no v3 import): `python3 tests/http/migration_rehearsal.py live-rehearsal --v2-binary <path-to-pinned-skrynia-server> [--evidence-dir <dir>]`
- Future v2-vs-v3 differential (when disposable candidate exists): `python3 tests/http/differential_parity.py --inventory tests/http/v2_compat_inventory.json --differential-scenarios --v2-binary <pinned-v2> --candidate <http://disposable-v3-facade> [--candidate-root-id ... --candidate-root-issuer ...]`


## Continuation (2026-11-18, agent-mvAwg4Z28px41EsZMm9THD-honored, host marceline-dev)

**Scope:** Fresh disposable verification on pinned v2 baseline only (no candidate v3+facade). No code changes to harness; evidence gathered from existing PR #11 branch state (`c69d127` at run start) and recorded for board/review. **No production data mutated. No v2-vs-v3 parity claimed.**

**Pinned baseline used:** Skrynia `10cbf0d5ad00224e1375a4f2482965f8d3513559` debug binary `skrynia/target/debug/skrynia-server` (rustc/cargo 1.93.0) in isolated clone `/tmp/opencode/skrynia-parity-jcsZAxSeTC0x4SsKXOKohF-screwed-2ntGmTZ0qiM98ky9Q7vA43/skrynia`.

**Executable disposable results (all PASS, rc=0):**
- `python3 -m py_compile tests/http/differential_parity.py tests/http/migration_rehearsal.py` OK.
- `python3 tests/http/differential_parity.py --self-test` => `self_test_ok generated_cases=43`.
- `./tests/http/v2_compat_harness.sh check` => `inventory_ok endpoints=17 corpus=12`.
- `--baseline-self-consistency --v2-binary <v2bin> --json-out .../self_consistency.json` => **pass**, scenarios=5, observations=50, failures=0: `public_meta`(3), `store_reads_writes_conditionals_ranges_auth`(28), `grants`(9), `quota`(4), `push_validation`(6). (v2-vs-v2 same pinned build on fresh disposable data dirs; not parity proof.)
- `--concurrency --v2-binary <v2bin>` => **ok:true**, 7 checks: concurrent creates 3 rounds each 1x201/7x409, concurrent CAS replace 1x200/7x412, concurrent issue/revoke users consistent, post-concurrency health 200.
- `--crash-restart --v2-binary <v2bin>` => **ok:true**, 5 checks: committed write durable after kill+restart, continued writes after restart, 20/20 acknowledged burst writes readable after restart, post-crash health 200.
- `--security --v2-binary <v2bin>` => **ok:true**, 16 checks: traversal/encoded traversal denied without leak (400), malformed/missing auth denied (401), malformed grant bodies rejected (400), issuer cannot write store/user cannot issue users/out-of-scope write denied (403), auth errors do not echo bearer token.
- `--latency-writes --v2-binary <v2bin> --latency-warmup 20 --latency-count 200 --latency-payload-bytes 128` => **ok:true** (v2 baseline only). Methodology: POST create-if-absent unique keys, payload 128 bytes, warmup 20, count 200, concurrency 1, localhost HTTP, disposable local fs, includes request/response overhead. Results (ms): p50=11.046648, p95=12.219709, p99=16.315418, min=10.462156, max=17.459923, mean=11.227682. **No candidate v3+facade comparison.**
- `migration_rehearsal.py live-rehearsal --v2-binary <v2bin> --evidence-dir .../migration` => **ok:true**, `production_mutation:false`: populated 3 objects (incl. nested key) + 1 child grant, snapshot/manifests matched source==rollback==import-candidate (sha256 `8a71b7a7a36e979c04447b479e56b376e6b17f3c0d63c1f885e4c508fc0deaea`), original restarted and reads verified, rollback copy untouched; `v3_import_and_post_import_parity` **not_run** (no integrated candidate).

**Exact pass/fail summary (disposable v2-side executable suites this run):** self-test 1/1 pass (43 generated cases); inventory/harness check 1/1 pass (17 endpoints, corpus 12); baseline scenario observations 50/50 pass across 5 scenarios (0 failures); concurrency checks 7/7 pass; crash/restart checks 5/5 pass; security probes 16/16 pass; latency baseline measurement 1/1 pass (200 samples, 0 write errors); migration rehearsal 4/4 executed steps pass + 1 step explicitly not_run (v3 import). **v2-vs-candidate differential executed: 0 scenarios (blocked).**

**Blocked cases / not yet proven (precise):**
- No complete integrated v3 kernel + Program 0 + skrynia-tools compatibility facade candidate runnable on host for end-to-end differential (`--differential-scenarios` harness mode available but not executed). Observed partial components insufficient for full parity: kernel workdirs/binaries exist but no proven integrated Program0+facade serving full v2 wire protocol; no Program 0 `.wasm` artifact identified in searched `/tmp/opencode` program0 paths; uTJ facade scaffold limited to health/version/push_vapid/preflight.
- v2-vs-v3 warm write p50/p95/p99 comparison not executed (candidate unavailable).
- Actual v2→v3 import/export into candidate and post-import parity/rollback against v3 not executed (v3 import step not_run).
- Not executed: real HTTPS push delivery/ack/end-to-end subscription lifecycle, reverse-proxy/browser/TLS compatibility, full exhaustive error-status matrix beyond implemented scenarios/probes, Skrynia/skrynia-tools full `cargo test`/clippy/CI in this run, independent review, reproducible deploy/cutover, production rollback drill.
- No evidence supports release/cutover or removal of v2 Rust business logic; PR #11 must remain draft until full proven gate.

**Local evidence artifacts (outside repo):** `/tmp/opencode/parity-evidence-mvAwg4Z28px41EsZMm9THD-2ntGmTZ0qiM98ky9Q7vA43/{self_test.txt,v2_compat_harness_check.txt,self_consistency.{log,json},concurrency.{log,json},crash_restart.{log,json},security.{log,json},latency_v2.{log,json},migration.log,migration/*}`.
