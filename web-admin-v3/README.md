# Skrynia v3 native web administration (static app)

Independent, static browser administration front-end for Skrynia v3 native
Program 0 deployments. This directory is a web application published as ordinary
static assets on a trusted HTTPS static host/origin (for example
`https://tools.a.vau.place`). It does **not** provide an admin CLI, server-side
admin daemon, or any legacy v2 HTTP facade.

## Status and backend dependency

- **Native Program 0 ABI not finalized here.** Backend registry auth/rollback
  protocol is tracked separately (Antonina issue
  `7X0twPgWYhSfBcCUBTZlwS-poor`, agent `TsmxG6zM7rBp5wl8lN1R8V-holder`). This UI
  does **not** invent or hardcode an unfinalized wire format.
- **Fail-closed by default.** Real operations require an injected, tested
  semantic capability adapter. Without it the UI reports unavailable operations
  and performs no privileged action.
- **No v2 compatibility:** no `/store`, `/grants`, GET/HEAD/PUT/DELETE admin
  facade, or other v2 HTTP routes are used.

## Security model (UI vs server)

- The browser UI and fake in-memory adapter **do not** provide security,
  authorization, or readiness enforcement. They are for local logic/testing and
  operator interaction only.
- **Server-side authorization is required** for every privileged operation
  (listing protected admin state, reading manifests/state as applicable, and
  activation/rollback). The injected adapter/backend must authenticate and
  authorize the operator request using the frozen native protocol; this UI must
  not be treated as a security boundary.
- **Server-side readiness verification is required** before activation. The
  backend/Program 0 must verify that the selected release is actually READY,
  valid, and eligible to become active; the UI may pre-check loaded state but
  cannot prove readiness or integrity on its own.
- **Trusted operator secret:** if the adapter requires an operator credential,
  the UI keeps it in browser session memory only while unlocked. It is not
  embedded in static assets, not placed in URLs/query strings/fragments, not
  persisted to local storage or the encrypted vault by this UI, and not logged
  by this UI. It is cleared on lock. The injected adapter is responsible for not
  leaking it.
- **Encrypted browser vault:** local vault data is encrypted client-side
  (PBKDF2-SHA256 + AES-256-GCM, context bound as AAD) for convenience/storage of
  local admin state. Vault unlock does not authenticate to the server.
  `navigator.locks` is required for atomic local vault CAS updates.

## Capabilities and UX

The UI is designed around a semantic capability adapter:

- `listApps(): Promise<string[]>`
- `getAppState(app): Promise<{name, active, pointerVersion, versions:[{id, ready, usable, error?, commit?, repo?, files?}]}>`
- `getReleaseManifest(app, releaseId): Promise<normalized manifest>`
- `activateRelease(app, releaseId, {expectedActive, expectedPointerVersion}): Promise<{app, active, pointerVersion}>`
- optional `setOperatorSecret(secret: string): void`

`window.__skryniaV3AdminCapabilities` may be injected as an object or a factory
returning such an adapter.

Implemented UI behavior:

- Lists discovered applications and **all** historical/ready releases loaded by
  the adapter, with version IDs/hashes, ready/usable status, active pointer,
  and loaded optimistic CAS pointer state.
- Handles large histories (e.g. 1000 versions) with client-side pagination and
  navigation while keeping all versions accessible.
- Manifest inspection for historical releases.
- Activation/rollback to any selected ready/usable release via optimistic
  compare-and-swap using expected active release and expected pointer state
  (`expectedActive`, `expectedPointerVersion`) from the loaded app state.
  Activation is intended to be O(1) in version count and asset bytes as a
  pointer/state transition only; no release assets or application data are
  copied or migrated by the UI.
- **Conflict refresh:** CAS conflicts are reported explicitly and state is
  refreshed.
- **Lost-ack reconciliation:** if an activation outcome is uncertain/unknown
  (e.g. transport/lost response), the UI marks the app as requiring
  reconciliation, blocks further activations for that app, and requires an
  explicit refresh/reconcile of the current active pointer before continuing.
  Uncertain mutations are never automatically retried.

## Transport

`binary_transport.js` provides a low-level binary POST helper
(`application/octet-stream`, POST-only). Actual request/response framing,
authorization, and protocol semantics must be implemented only by a tested
adapter once the native Program 0 ABI is concrete and evidenced.

## Static artifact packaging

`web-admin-v3/` is a self-contained static front-end (HTML/CSS/ES modules). An
isolated static artifact can be produced by copying the directory contents as-is
to a publishable static bundle (no server-side rendering or admin daemon).
Example:

```sh
rm -rf dist/web-admin-v3 && mkdir -p dist/web-admin-v3
cp web-admin-v3/*.html web-admin-v3/*.css web-admin-v3/*.js web-admin-v3/README.md dist/web-admin-v3/
```

Serve over HTTPS from the intended static origin with correct content types and
cache policy appropriate for release immutability. Preserve origin routing to
the native Program 0 endpoint as defined by deployment; this UI does not assume
or create v2 routes.

## PWA / origin routing

No service worker is bundled by default in this draft. If a PWA/service worker is
added for this origin, it must not cache or replay privileged POST admin
operations, must not store operator secrets, and must preserve same-origin/admin
routing semantics. Any PWA behavior requires separate validation.

## Tests

Run from repository root:

```sh
node tests/web-admin-v3/crypto_vault.mjs
node tests/web-admin-v3/app_history_activation.mjs
node tests/web-admin-v3/fail_closed.mjs
node tests/web-admin-v3/browser_vault_lock.mjs
node tests/web-admin-v3/pagination_reconcile.mjs
node tests/web-admin-v3/session_and_adapter.mjs
```

Tests cover fake app history, manifest inspection, O(1)-style pointer CAS
activation/rollback in both directions, conflicts, invalid/unready handling,
vault crypto/session/locking behavior, fail-closed defaults, pagination logic,
uncertain/lost-ack reconciliation behavior, and session/operator-secret adapter
contract expectations. They do **not** prove server-side authorization,
readiness enforcement, or a finalized native wire protocol.

## Native release identity and ABA safety

Version identifiers are 64 lowercase hexadecimal digits (SHA-256), matching the
native static release identity model expected by this draft. Activation requires
both expected active release and expected pointer revision/state so A→B→A cannot
satisfy a stale request by release ID alone. A real adapter/backend must provide
durable, bounded revision/nonce semantics; the in-memory fake only validates the
UI/contract behavior.
