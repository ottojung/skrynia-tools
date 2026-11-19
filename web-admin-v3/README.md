# Skrynia v3 native web administration (front-end)

Independent, static browser administration front-end for Skrynia v3 native
Program 0 deployments. It is intended to be hosted as ordinary read-only static
assets on a trusted origin such as `https://tools.a.vau.place` (static-host
infrastructure is tracked separately; this directory contains only the app).

## Status

- **Transport/protocol:** fail-closed by default. The admin communicates solely
  over an injected capability adapter for explicit native binary POST Program 0
  operations once the versioned, authorized protocol is frozen. This build does
  **not** guess or implement unfrozen native opcodes/auth/wire formats.
- **No v2 compatibility:** no `/store`, `/grants`, GET/HEAD facade, or other v2
  HTTP routes are used.
- **Capabilities implemented in front-end logic:** list all applications, list
  all historical/ready releases, inspect historical release manifests, and
  atomically activate or roll back to any ready release by compare-and-swap of a
  single active-release pointer only (O(1) in version count and asset bytes).
  Conflicts are reported explicitly; no release assets or application data are
  copied or migrated.
- **Vault:** encrypted local/browser vault (PBKDF2-SHA256 + AES-256-GCM, vault
  context bound as AAD). The access secret is never sent to a server, stored in
  public release assets, or logged. Credential/authority model details beyond
  encrypted local storage require the frozen native protocol and are not guessed.
- **Testing:** deterministic fake-capability tests cover app discovery/history,
  manifest inspection, O(1) pointer CAS activation/rollback in both directions,
  conflict and invalid-state handling, vault crypto/session behavior, and
  fail-closed defaults.

## Integration contract (pending)

`window.__skryniaV3AdminCapabilities` may be injected as an object or factory
returning a semantic capability adapter:

- `listApps(): Promise<string[]>`
- `getAppState(app): Promise<{name, active, pointerVersion, versions:[{id, ready, usable, error, commit, repo, files}]}>`
- `getReleaseManifest(app, releaseId): Promise<normalized manifest>`
- `activateRelease(app, releaseId, {expectedActive}): Promise<{app, active, pointerVersion}>`

A future native adapter will translate these operations to the frozen,
authorized binary POST protocol for the operator-supplied Program 0. Until then
the UI remains fail-closed.

## Tests

Run from repository root:

```sh
node tests/web-admin-v3/crypto_vault.mjs
node tests/web-admin-v3/app_history_activation.mjs
node tests/web-admin-v3/fail_closed.mjs
```
