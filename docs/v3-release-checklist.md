# Skrynia v3 parity, migration and release checklist (draft)

Status: **draft working document** for milestone `dgKBALkpO7M7oKBHrZUzCT-shortly`, issue `2ntGmTZ0qiM98ky9Q7vA43-running`.
Scope: black-box differential parity gates, safe v2→v3 data migration/rollback validation, and release readiness evidence. This document does not authorize production mutation.

## Hard constraints

- **No production data mutation** by tests, harnesses, or migration rehearsal unless an explicitly reviewed runbook step authorizes it on a disposable copy.
- **No CLI in skrynia-tools**. Administration and deploy remain web/scripted per project rules; migration validation must not introduce operator CLI commands.
- **v3 kernel boundary**: one public kernel endpoint `POST /` with `Content-Type: application/octet-stream`, request body passed unmodified to Program 0. Legacy HTTP routes/methods/status/header semantics are provided only by the skrynia-tools compatibility HTTP facade; parity is measured at the facade wire boundary, not claimed for the kernel alone.
- **Pinned v2 baseline**: every differential claim must record the exact v2 server source commit/build artifact, skrynia-tools commit, configuration, and data snapshot provenance. Candidate pin for v2 Rust server implementation: Skrynia `feat/rust-server` commit `c7b527e` (`/workspace/skrynia-rust-rewrite` observation) — confirm canonical pin against deployed/release baseline before gate sign-off.
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

## Evidence log (to be filled)

| Date | Component | Commit/artifact | Command | Result | Notes |
| --- | --- | --- | --- | --- | --- |
| 2026-11-18 | parity harness draft | skrynia-tools (this branch) | `python3 tests/http/differential_parity.py --self-test` | pending | |
