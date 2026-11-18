# Pinned v2 HTTP compatibility inventory (phase 4)

Status: **inventory + harness scaffold** for milestone `dgKBALkpO7M7oKBHrZUzCT-shortly`, issue `uTJ5794ggJJa1wpNnOSc9V-teacher`.
Repository: `ottojung/skrynia-tools`.
Pinned skrynia-tools reference commit: `aba366b` (`fix: preserve app URL query and fragment across version redirect`).

This document enumerates the observable v2 grant-based wire protocol that the
skrynia-tools compatibility facade must preserve exactly when translating to
Program 0 `POST /` on the v3 VM. It is derived from the pinned skrynia-tools
client/test corpus (`tests/http/protocols_live.sh`, `tests/http/deploy_live.sh`,
`tests/http/revoke_live.py`, native client behavior under `src/client/*`,
`src/cas/*`, `src/event_set/*`, `src/log/*`) and the v2 API/protocol material
currently captured in `skrynia-v3-spec/docs/api-spec.md` and
`skrynia-v3-spec/docs/cookbook.md` (milestone workspace). Items marked
**source-needed** require confirmation against the authoritative deployed v2
server behavior / pinned v2 server commit before claiming parity.

## Scope and non-goals

- Preserve exact legacy HTTP routes, methods, headers, status codes, error
  codes/bodies, and observable quirks through a skrynia-tools compatibility
  HTTP facade. The v3 kernel remains single-endpoint `POST /`.
- No CLI commands or admin CLI in skrynia-tools; web-only admin and deploy
  script remain. No root credentials exposed.
- v2 data migration is separate and explicit; never alter production data.
- Differential parity against pinned v2 baseline is the completion gate; this
  inventory is a prerequisite, not proof of compatibility.

## Global wire conventions

| Area | Observable behavior | Sources / notes |
| --- | --- | --- |
| Base URL | Paths are relative to `SKRYNIA_URL` including optional path prefix. Clients reject query/fragment/userinfo and require http/https. | `src/client/types.rs`, deploy/protocol live scripts. |
| Auth | Mutations/listing/grant ops use `Authorization: Bearer <token>` unless noted. Issuer vs user tokens are distinct capabilities. `X-Skrynia-Capability` used for push subscription update/delete. | api-spec, `src/client/grants.rs`, `src/client/revoke.rs`, browser fetch. |
| Credentials | Never expose root/issuer tokens in public objects, URLs, logs, normal JSON output. Root issuer stored privately server-side. | AGENTS, security docs, api-spec. |
| CORS | `Access-Control-Allow-Origin: *`; preflight allows `X-Skrynia-If-SHA256`, `If-None-Match`, `If-Range`, `Range`; exposes ETag. No cookie auth. | api-spec. **source-needed**: exact preflight methods/headers/status. |
| Content types | Object GET serves bytes with MIME by extension (`.html .js .mjs .css .json .svg .png .wasm`, else `application/octet-stream`), `Cache-Control: no-cache`, `X-Content-Type-Options: nosniff`. JSON APIs return JSON bodies. | api-spec. **source-needed**: exact Content-Type/charset for JSON/error responses. |
| Errors | JSON error objects include `error` code in observed client handling (e.g. `grant_quota_exceeded`, `user_not_found`, `grant_not_found`, `content_mismatch`, `invalid_precondition`, `path_conflict`). | client error parsing, revoke_live, api-spec. **source-needed**: full error schema, message fields, all codes per endpoint. |
| Success contract | Clients treat unexpected 2xx as contract violations (not committed): create expects 201, confirmed mutation/grant/revoke expect 200, complete GET/list expect 200. 202/204/206 not proof of committed/complete result. | `docs/security.md`, `src/client/errors/status_test.rs`. |

## Public/meta endpoints

| Method | Path | Auth | Success | Negative / notes |
| --- | --- | --- | --- | --- |
| GET | `/health` | public | 200 (healthy). | 503 when recovery latch set per ops docs. **source-needed**: body format. |
| GET | `/version` | public | 200. | **source-needed**: body/fields. |
| GET | `/push/vapid` | public | 200, returns server public VAPID key. | **source-needed**: JSON shape (`publicKey`/other), errors. |

## Object store endpoints

Object key grammar: nonempty ASCII path <=255 bytes, segments `[A-Za-z0-9_-][A-Za-z0-9_.-]*`, no leading dot/empty/`.`/`..`, no `%`, no `*`, no trailing/leading slash, extensions significant. Matchers: exact key, `prefix*`, `prefix/*`, `*`, `""` (none), max 257 bytes.

| Method | Path | Auth | Semantics | Success | Negative / observable errors |
| --- | --- | --- | --- | --- | --- |
| GET | `/store/{key...}` | public | Raw bytes, strong `ETag: "<sha256>"`, conditional/range reads. `Cache-Control: no-cache`, `X-Content-Type-Options: nosniff`. | 200 full body; 206 single range with `Content-Range`; 304 `If-None-Match` without body. | 404 absent. 416 unsatisfiable range. Invalid key/path escape: **source-needed** (400 expected). |
| HEAD | `/store/{key...}` | public | Metadata only, same validators as GET. | 200 (headers, no body). 304 on matching validator where applicable. | 404 absent. **source-needed**: parity of conditional handling with GET. |
| GET | `/store?locations={matcher}` | `Authorization: Bearer <user-token>` with `list` on covering scope | List matching keys, sorted, all stored objects matching regardless of creator grant. Pagination `limit` 1-1000, `after` exclusive cursor, default/max page 1000, non-final page includes `next`. | 200 JSON `{"keys":[...]}` and optional `next`. | 400 missing/duplicated/invalid matcher. 403 matcher outside authority or missing `list`. Anonymous unavailable. **source-needed**: exact 401/403 split, empty result shape. |
| POST | `/store/{key...}` | Bearer user token with `create` | Create if absent with initial payload. Rejects `If-Match/If-None-Match/If-Modified-Since/If-Unmodified-Since/If-Range` and `X-Skrynia-If-SHA256`. | 201. 409 is create collision (exists). | 409 `path_conflict` for file/dir collision. 400 `invalid_precondition` for forbidden precondition headers. 403 insufficient authority/location/quota as applicable. 507 if mutation would need new outbox entries and outbox full (push interaction). 503 upload backpressure. **source-needed**: body on 409 exists vs path_conflict. |
| PUT | `/store/{key...}` | Bearer user token with `modify` | Replace existing key. Optional `X-Skrynia-If-SHA256` (64 lowercase hex) checked and mutated as one synchronous step. Rejects HTTP conditional headers listed for mutations. | 200. | 404 absent. 412 `content_mismatch`. 400 `invalid_precondition` (bad/misused precondition). 403/507/503 as applicable. |
| DELETE | `/store/{key...}` | Bearer user token with `delete` | Delete existing key. Optional `X-Skrynia-If-SHA256` supported. Rejects HTTP conditional headers listed for mutations. | 200. | 404 absent. 412 `content_mismatch`. 400 `invalid_precondition`. 403 as applicable. |

### Reads/conditionals/ranges/ETags (detail)

- ETag is strong SHA-256 of object bytes: `ETag: "<sha256>"`.
- `If-None-Match` on GET/HEAD yields 304 without body when matching.
- Single `Range: bytes=...` including suffix ranges yields 206 + `Content-Range`; unsatisfiable yields 416.
- Matching strong `If-Range` authorizes partial response; stale validator returns full current object.
- Byte-identity ABA: A→B→A restores digest; CAS/history not inferred.
- Atomicity: per-key write commit; failed conditional mutation preserves existing object. No multi-key ACID.
- Uploads streamed to fsynced staging with incremental SHA-256; inactivity timeout between chunks; `SKRYNIA_MAX_OBJECT_SIZE` default 10GB; `SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES` default 40MiB; `SKRYNIA_MAX_CONNECTIONS` default 512.

## Grant and credential endpoints

Tree of grants: `id`, `parent` (null root), `locations`, `operations`, `maxBytes/maxObjects`, issuer credential, zero+ revocable user credentials. Operations: `create, modify, delete, list, delegate, subscribe`. Root auto-created on startup if absent with `locations:"*"`, all ops, unlimited null limits; root issuer token local-only/private 0600, never via HTTP.

| Method | Path | Auth | Semantics | Success | Negative / notes |
| --- | --- | --- | --- | --- | --- |
| POST | `/grants` | Bearer parent user token with `delegate` | Create child grant. Request requires explicit string `locations`, array `operations`, positive safe-integer `maxBytes`, `maxObjects`; child narrows only. Response contains new grant ID and one-time issuer token, **no user token**. | 200 (per client contract/tests). | 400 missing/wrong/duplicate/unknown fields or invalid narrowing. 403 no delegate/insufficient scope/quota. **source-needed**: exact status/error codes. |
| GET | `/grants/{id}` | Bearer exact grant user token | Inspect grant and current usage. | 200 JSON grant/usage. | 401/403/404 semantics **source-needed**. |
| POST | `/grants/{id}/users` | Bearer exact issuer token | Issue user token. Returns `grantId`, `userId`, `userToken`. | 200. | 404 `grant_not_found` or revoked/missing grant. 403 wrong credential type. **source-needed**. |
| DELETE | `/grants/{id}/users/{userId}` | Bearer exact issuer token | Revoke one user token. Revocation invalidates token but never deletes object values. | 200. | 404 `user_not_found` or `grant_not_found` treated as confirmed absence only with expected code (client behavior). 403 wrong issuer. |
| DELETE | `/grants/{id}` | Bearer exact issuer token | Revoke grant subtree (tokens, sponsored push subscriptions) but never delete object values. | 200. | 404 `grant_not_found` confirmed only with expected code. |

### Quotas/accounting (exact formulas from api-spec)

For grant g:
- `bytesUsed(g) = sum(current matching objects of payloadBytes+512) + 2048*grants_in_inclusive_descendant_tree + 128*extra_user_tokens_beyond_first_in_tree + 4096*sponsored_push_subscriptions_in_tree`
- `objectsUsed(g) = count(matching stored objects) + count(sponsored push subscriptions in descendant tree)`
- Only acting grant and ancestors checked on creation/modification; unrelated grants cannot veto. PUT charged as full resulting value with old removed first; DELETE needs no quota headroom. Grant/user credential creation consumes ancestor metadata budget. Quota exhaustion surfaced explicitly (observed `grant_quota_exceeded`, 403 in client parsing context).

## Web Push / subscriptions

| Method | Path | Auth | Semantics | Success | Negative / notes |
| --- | --- | --- | --- | --- | --- |
| GET | `/push/vapid` | public | Server public VAPID key. | 200. | **source-needed**. |
| POST | `/push/subscriptions` | Bearer user token with `subscribe` | Register subscription. Body: `locations`, `events` optional default `[create,replace,delete]`, `endpoint` HTTPS, `keys{p256dh,auth}`. Returns `id`, `capability`, `deduped`. | 200/201 **source-needed**. | 400 invalid body/events/endpoint/keys. 403 missing subscribe or scope widening. 507 outbox-full rejection can affect committing mutations needing new entries. **source-needed**: exact statuses. |
| PUT | `/push/subscriptions/{id}` | `X-Skrynia-Capability` | Change endpoint/keys only; locations/events immutable. | 200 **source-needed**. | 404/403 invalid capability. |
| DELETE | `/push/subscriptions/{id}` | `X-Skrynia-Capability` | Remove subscription. | 200 **source-needed**. | 404/403. |

Delivery semantics: durable retryable wake-ups coalesced to one pending per subscription per mutation batch; payload constant `changed`, no object data; duplicate wake-ups possible; exponential backoff, removed after 12 failures default; no push host allowlist; outbound DNS validated/pinned to public routable addresses; bounded concurrency/timeouts. Grant revocation removes sponsored subscriptions.

## Differential harness scaffold (this increment)

- `tests/http/v2_compat_inventory.json`: machine-readable fixture catalog pinned to this inventory/commit for endpoint grammar, headers, status/negative cases, and corpus references.
- `tests/http/v2_compat_harness.sh`: scaffold to validate inventory structure and, when `SKRYNIA_V2_BASE_URL` (and optional credentials/fixtures) are provided, run differential probes against a target server/facade. It must not assume a live server in CI by default and must not mutate production data.

## Known gaps / next increments

- **Pinned v2 server baseline**: obtain and record the authoritative deployed v2 server commit/binary and canonical reference test corpus; resolve all **source-needed** fields from it.
- Encode executable black-box cases for every row above (public reads/HEAD/range/ETag/If-*, listings/pagination, grants/issuer/user issuance/revocation, quotas, push subscribe/update/delete/delivery/ack behavior as observable, and all negative/error cases).
- Implement compatibility HTTP facade in skrynia-tools translating legacy routes to Program 0 `POST /` with injected capabilities (no CLI/admin CLI, web-only admin preserved).
- Run differential parity against pinned v2 baseline, including concurrency/crash/reload/security/quota cases per milestone; record deviations as bugs.
- Coordinate with 01-03 for VM/Program 0 ABI readiness before claiming facade completeness.
