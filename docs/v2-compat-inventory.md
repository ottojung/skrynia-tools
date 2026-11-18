# Pinned v2 HTTP compatibility inventory (phase 4)

Status: **inventory + initial facade scaffold** for milestone
`dgKBALkpO7M7oKBHrZUzCT-shortly`, issue `uTJ5794ggJJa1wpNnOSc9V-teacher`.
Repository: `ottojung/skrynia-tools`.
Pinned skrynia-tools reference commit: `aba366b` (inventory baseline).

This document enumerates the observable v2 grant-based wire protocol that the
skrynia-tools compatibility facade must preserve exactly when translating to
Program 0 `POST /` on the v3 VM. It is derived from pinned skrynia-tools client
and test corpus and from the authoritative pinned v2 server implementation.

## Pinned baselines

| Component | Pin | Evidence |
| --- | --- | --- |
| skrynia-tools reference | `aba366b` (`fix: preserve app URL query and fragment across version redirect`) | issue handoff, inventory commit base |
| v2 server baseline | `ottojung/Skrynia` `a33336e29c8781115489a3b4adc270573bc458fc` (`docs: specify hierarchical keys...`) | `docs/protocols.md`, `.github/workflows/ci.yml` in skrynia-tools pin CI to this commit; source inspected at this commit for route/error/response behavior |
| v3 Program 0 ABI | **coordination pending** (issues 01-03) | facade uses injected backend trait; kernel exposes single `POST /` octet-stream invoke (observed in v3 kernel repos) |

Items still marked **source-needed** require confirmation against running pinned
v2 server differential traces or Program 0 ABI publication.

## Scope and non-goals

- Preserve exact legacy HTTP routes, methods, headers, status codes, error
  JSON `{"error":...}` codes, and observable quirks through a skrynia-tools
  compatibility HTTP facade. The v3 kernel remains single-endpoint `POST /`.
- No CLI commands or admin CLI in skrynia-tools; web-only admin and deploy
  script remain. No root credentials exposed.
- v2 data migration is separate and explicit; never alter production data.
- Differential parity against pinned v2 baseline is the completion gate.

## Global wire conventions (pinned baseline)

| Area | Observable behavior | Evidence |
| --- | --- | --- |
| Base/mount | Paths relative to `SKRYNIA_URL` prefix; mounted prefix stripped in dispatch. | v2 `http/entry/common.rs`, client URL handling. |
| Auth | Mutations/listing/grants use `Authorization: Bearer <token>`; push update/delete use `X-Skrynia-Capability`; push subscribe may use bearer. | v2 routes/replies, client code. |
| CORS | `Access-Control-Allow-Origin: *`, `Access-Control-Expose-Headers: ETag`. Preflight 204 with `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`, `Access-Control-Allow-Headers: Authorization, Content-Type, X-Skrynia-Capability, X-Skrynia-If-SHA256, If-None-Match, If-Range, Range`, `Access-Control-Max-Age: 3600`. | v2 `http/replies/mod.rs`. |
| JSON responses | JSON replies include `Cache-Control: no-store` when content-type is application/json and not already set. | v2 `replies::cors`. |
| Errors | Axum JSON error bodies: `{"error":"<code>"}`. | v2 `error.rs`, handlers. **source-needed**: exhaustive per-endpoint code/message parity under all failure modes. |
| Recovery latch | When server failed state set, dispatch returns 503 `{"error":"recovery_required"}`. | v2 `entry/common.rs`. |
| Store mutation success | POST create returns 201 `{"ok":true}` or `{"ok":true,"notificationPending":true}`; PUT/DELETE return 200 same JSON shape. | v2 `http/write/mod.rs`. |
| Public utility | GET `/health` => 200 `{"ok":true}`. GET `/version` => 200 build info JSON. | v2 `http/entry/common.rs`, `validate_build_info`. **source-needed**: exact version JSON fields. |
| Push VAPID | GET `/push/vapid` => 200 `{"publicKey":"..."}`. | v2 `push/api/routes/mod.rs`. |

## Object store endpoints

Object key grammar and matcher rules as in v2 api-spec (ASCII path <=255, segment
rules, no traversal/%, `*` only in matcher positions, max matcher 257).

| Method | Path | Auth | Semantics | Success | Negative / observable errors |
| --- | --- | --- | --- | --- | --- |
| GET | `/store/{key...}` | public | Raw bytes, strong ETag, conditional/range reads. | 200/206/304 | 404/416/400 invalid key. **source-needed**: exact header set/MIME/cache parity. |
| HEAD | `/store/{key...}` | public | Metadata only. | 200/304 | 404. **source-needed**: conditional parity details. |
| GET | `/store?locations={matcher}` | bearer user with `list` and subset scope | Sorted keys, optional `limit`/`after`, `next` on non-final page. | 200 JSON with `keys`, optional `next` | 400 invalid/missing/duplicated matcher, 403 no list or outside scope, auth errors. **source-needed**: exact shapes. |
| POST | `/store/{key...}` | bearer user `create` | Create if absent. Rejects mutation conditionals incl. `X-Skrynia-If-SHA256`. | 201 JSON | 409 exists/path conflicts, 400 invalid precondition/json/body issues as applicable, 403, 507 insufficient storage/outbox-full cases, 503 transient/backpressure. **source-needed**: exact split. |
| PUT | `/store/{key...}` | bearer user `modify` | Replace existing, optional `X-Skrynia-If-SHA256`. | 200 JSON | 404, 412 content mismatch, 400, 403, 507, 503. |
| DELETE | `/store/{key...}` | bearer user `delete` | Delete existing, optional `X-Skrynia-If-SHA256`. | 200 JSON | 404, 412, 400, 403. |

## Grant and credential endpoints

| Method | Path | Auth | Semantics | Success | Negative / notes |
| --- | --- | --- | --- | --- | --- |
| POST | `/grants` | bearer parent user with `delegate` | Create child grant from JSON spec. | 200 JSON (grant id + issuer token, no user token) | 400 invalid json/fields, 403, quota/authority errors. **source-needed**: exact body fields/codes. |
| GET | `/grants/{id}` | bearer exact grant user | Inspect grant/usage. | 200 JSON | 401/403/404 **source-needed**. |
| POST | `/grants/{id}/users` | bearer exact issuer | Issue user token. | 200 JSON `grantId,userId,userToken` | 404 `grant_not_found`, 403 wrong credential. |
| DELETE | `/grants/{id}/users/{userId}` | bearer exact issuer | Revoke user. | 200 `{"ok":true,"grantId","userId"}` | 404 `user_not_found`/`grant_not_found`, 403. |
| DELETE | `/grants/{id}` | bearer exact issuer | Revoke grant subtree. | 200 `{"ok":true,"grantId"}` | 404 `grant_not_found`, 403. |

Quota/accounting formulas follow v2 api-spec and must match pinned server behavior.

## Web Push / subscriptions

| Method | Path | Auth | Semantics | Success | Negative / notes |
| --- | --- | --- | --- | --- | --- |
| GET | `/push/vapid` | public | Returns `{"publicKey":...}`. | 200 | **source-needed**: absent/unconfigured cases. |
| POST | `/push/subscriptions` | bearer user `subscribe` (and/or dedup capability path) | Register subscription. | 200 JSON with `id,capability,deduped` (observed) | 400/403/507. **source-needed**: exact validation/status matrix. |
| PUT | `/push/subscriptions/{id}` | `X-Skrynia-Capability` | Change endpoint/keys only. | 200 **source-needed** | 400/403/404. |
| DELETE | `/push/subscriptions/{id}` | `X-Skrynia-Capability` | Remove subscription. | 200 **source-needed** | 403/404. |

Delivery/ack semantics remain **source-needed** for executable differential
proof; outbox/coalescing/retry behavior per api-spec.

## Artifacts in this increment

- `tests/http/v2_compat_inventory.json`: machine-readable catalog pinned to
  baselines above for endpoint grammar, statuses, headers, and corpus refs.
- `tests/http/v2_compat_harness.sh`: inventory validation and safe probe modes.
- `src/wire/v2compat/*`: initial facade scaffold with injected backend
  capability trait and deterministic unit tests (foundation for parity work).

## Gaps / next increments

- Publish/confirm Program 0 ABI for v2-compat operations and wire full
  translation path over kernel `POST /`.
- Resolve remaining **source-needed** fields by differential runs against an
  isolated build of pinned v2 server `a33336e...` and expand harness cases.
- Implement complete HTTP facade routing/auth/header/body/error parity for all
  endpoints, including ranges/conditionals, listing pagination, grants, quotas,
  and push delivery/ack observables.
- Run differential parity (including concurrency/crash/security/quota) and
  record deviations; coordinate with PR #11 harness/checklist.
