# Skrynia v3 native web administration (front-end)

Independent, static browser administration front-end for Skrynia v3. It is
hosted as ordinary read-only static assets on a trusted origin such as
`https://tools.a.vau.place` (static-host infrastructure is tracked separately;
this directory contains only the app).

## Architecture boundary (owner decision 2026-11-19/20)

- **Kernel:** generic binary `POST /` executor with two directional FDs. It
  knows only the operator-supplied root WASM; it implements no application
  protocol and has no special Program 1 or privileged root-FD path.
- **Program 0:** fixed operator-supplied dumb proxy only. It verifies a private
  256-bit bearer token, strips the 33-byte frame header and forwards the
  remaining bytes **opaque** to the operator-selected ordinary nonzero WASM
  child. It has no app registry, storage, grants, versions, rollback,
  administration or routing semantics and defines no root opcodes.
- **Ordinary nonzero WASM service:** all app versions, historical release
  manifests and the bounded compare-and-swap active pointer live here, behind a
  trusted app-scoped HTTPS gateway. The gateway holds the server-only Program 0
  token; the static browser assets never contain it.
- **No v2 compatibility:** no `/store`, `/grants`, GET/HEAD facade or other v2
  HTTP routes are used.

## Status

- **Transport/protocol:** fail-closed by default. The browser talks to the
  same-origin trusted gateway, not to Program 0; it sends opaque POST bytes and
  the gateway performs the pinned Program 0 token framing server-side. This
  build does not guess or implement native opcodes, admin instructions or token
  handling inside Program 0.
- **Capabilities implemented in front-end logic:** list all application IDs,
  list all sealed ready releases, inspect sealed release identity, and
  atomically activate or roll back to any ready release by compare-and-swap of a
  single fixed-size active pointer only (O(1) in version count and asset bytes).
  Conflicts are reported explicitly; no release assets or application data are
  copied or migrated. These operations are enforced by the compiled ordinary
  nonzero service, not by Program 0.
- **Vault:** encrypted local/browser vault (PBKDF2-SHA256 + AES-256-GCM, vault
  context bound as AAD). The access secret is never sent to a server, stored in
  public release assets or logged. It is a local UI secret, **not** the Program 0
  root bearer token. Authority for native operations comes from the ordinary
  service behind the gateway, not from the browser vault.
- **Testing:** deterministic fake-capability tests cover app discovery/history,
  manifest inspection, O(1) pointer CAS activation/rollback in both directions,
  conflict and invalid-state handling, vault crypto/session behavior, and
  fail-closed defaults. Injected fake capabilities are test doubles only: they
  do not demonstrate that a native Program 0, gateway or ordinary service
  backend exists or is authorized. Real kernel + Program 0 + service end-to-end
  qualification of this transport is provided by
  `tests/web-admin-v3/real_native_e2e/run.mjs` against pinned compiled artifacts
  (host-local HTTP, disposable token); see that harness README for exact scope.
  The native admin API gate is **closed**: `p1_admin_adapter.mjs` reports 42/42
  real-stack PASS over native opcodes 9-14. That evidence is pinned to earlier
  source heads (`3f92c133…` / `67212c89…`), so it is historical and **not**
  exact-head proof; the exact-head gate is the CI `compiled-stack` job.

## Integration contract (wired adapter)

`native_bootstrap.js` (loaded before `main.js`) installs
`window.__skryniaV3AdminCapabilities` from operator-provisioned runtime session
authority `window.__skryniaNativeSession = {endpoint, readCap, adminCap}`. With
no such session the shipped UI keeps its fail-closed default. The authority is
never stored in a static bundle, URL, release manifest or log; the trusted
static origin's reverse proxy adds the gateway edge capability server-side.

`native_capabilities.js` implements the semantic operations over the compiled
ordinary-P1 admin ABI through `createBinaryPostTransport`. Opcodes 6/7
(`AppPut`/`AppGet`) are **never** emitted; releases are no longer simulated with
client-side appdata JSON:

- `listApps()` sends `LIST_PROGRAMS` (14), paginated, and returns the sealed
  service's 32-byte application/invoke IDs as lowercase hex. There is no app
  listing among opcodes 9-13, so 14 is the native app-ID operation.
- `getAppState()` sends `GET_ACTIVE` (12) for the 42-byte live pointer record
  (`version=3 || live || 32-byte hash || u64 LE revision`) and `LIST_VERSIONS`
  (13), paginated, for all ready content hashes.
- `getReleaseManifest()` exposes an **identity-only** manifest: the compiled
  service stores sealed code hashes, not static file metadata, so no commit,
  repository or file list is fabricated. The UI labels it accordingly.
- `stageRelease()` / `sealRelease()` send `STAGE` (9) and `SEAL` (10) for the
  operator provisioning flow. An unstaged or unsealed hash is absent from the
  ready list and cannot be activated.
- `activateRelease()` issues exactly one `ACTIVATE` (11) frame: 32-byte admin
  capability, `u64 LE` expected revision and 32-byte target hash. The compiled
  service enforces the O(1) CAS on the fixed-size pointer, so A-B-A rollback to
  any earlier/later ready version advances the revision and rejects stale or
  replayed revisions. A malformed plan is rejected locally before the CAS.
  A lost ACK, gateway 5xx or malformed 2xx response is reported with
  `code: 'uncertain'`, distinct from a definitive `conflict`, `invalid_state`
  or `unauthorized` service/gateway denial. The controller blocks further
  activation for that app until an explicit refresh of authoritative state;
  it never auto-retries or spends a second CAS on an unknown outcome.

No Program 0 opcodes, v2 facade or CLI were added; Program 0 remains a dumb
token relay and the kernel remains generic. The trusted origin provisions the
32-byte invoke/read ID and admin capability out of band; optional
`session.adminCaps` maps invoke-ID hex to admin-capability hex for
multi-application administration. Root/issuer/operator/P0 tokens never enter
the browser.

`window.__skryniaV3AdminCapabilities` may be injected as an object or factory
returning a semantic capability adapter:

- `listApps(): Promise<string[]>`
- `getAppState(app): Promise<{name, active, pointerVersion, versions:[{id, ready, usable, error, commit, repo, files}]}>`
- `getReleaseManifest(app, releaseId): Promise<normalized manifest>`
- `activateRelease(app, releaseId, {expectedActive, expectedPointerVersion}): Promise<{app, active, pointerVersion}>`

A qualified adapter translates these operations into the authorized request
protocol of the **ordinary nonzero WASM service** and sends opaque bytes by
binary POST to the same-origin trusted gateway. The gateway, not this public
app, constructs the pinned Program 0 frame with its server-only token and
forwards it to the kernel. Never place a root, issuer or write token in public
JS, URLs or logs. If no adapter or gateway is configured the UI fails closed.

## P0 binary framing contract (pin 8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217)

The only Program 0 wire framing defined at this pin is:

```
POST body = 0x01 || 32-byte raw bearer token (SHA256-verified) || opaque payload
```

- `0x01` is the only accepted version byte; any other version is denied.
- The next 32 bytes are the raw secret. Program 0 reads the private
  `security/program0.sha256` (raw 32-byte SHA-256 digest, operator 0700/0600),
  computes `SHA256(raw)` and compares in constant time. A truncated header,
  wrong token, wrong-length or extra-byte verifier, or presenting the digest
  itself as the token are all denied.
- The remainder is **opaque**. Program 0 strips exactly 33 bytes and hands the
  rest to the operator-selected child; it interprets no opcodes and adds no app
  semantics. There are no root opcodes and no special Program 1 path.

`tests/web-admin-v3/p0_framing_decode.mjs` is a fail-closed reference decode
test for exactly this framing. It is a protocol-contract check with an injected
verifier and no embedded secret; it does not prove a live backend.

## Browser vault concurrency

Encrypted localStorage vault updates require the standard same-origin Web
Locks API. Every compare-and-swap save holds an exclusive lock across the
read/compare/write sequence; without Web Locks the UI rejects vault
initialization/unlock rather than pretending separate tabs are serialized.
Locking the vault invalidates its active session and clears UI password
fields, though JavaScript strings cannot be guaranteed to be physically
zeroized. Automated tests use a deterministic fake lock; actual multi-tab
browser behavior remains a release qualification gate.

## Tests

Run from repository root:

```sh
node tests/web-admin-v3/crypto_vault.mjs
node tests/web-admin-v3/app_history_activation.mjs
node tests/web-admin-v3/fail_closed.mjs
node tests/web-admin-v3/browser_vault_lock.mjs
node tests/web-admin-v3/p0_framing_decode.mjs
node tests/web-admin-v3/native_capabilities.mjs
node tests/web-admin-v3/uncertain_activation.mjs
```

The last two are deterministic wire-contract tests for the real admin opcodes
and fail-closed reconciliation; they use injected fakes and prove no backend.

Real native HTTP qualification against the pinned compiled kernel + P0 +
ordinary service + trusted gateway (host-local, not run by default):

```sh
node tests/web-admin-v3/real_native_e2e/run.mjs          # transport/P0 framing
node tests/web-admin-v3/real_native_e2e/p1_admin_adapter.mjs  # shipped adapter -> real P1 admin ABI
SKRYNIA_P1_WASM=/path/to/p1-83a65a2f.wasm node tests/web-admin-v3/real_native_e2e/p1_admin_adapter.mjs
```

Real Chromium UI-to-native end-to-end against the same pinned compiled stack,
from an independently hosted static origin (no simulator or fake adapter):

```sh
node tests/web-admin-v3/real_native_e2e/chromium.mjs
```
```

## Native release identity and ABA safety

Version identifiers are 64 lowercase hexadecimal digits (SHA-256), matching
the separate static-release packager and 32-byte native program identifiers.
Release manifests use the native schema skrynia-native-static-v1 with app,
release and files fields, not the withdrawn legacy application format.

Every activation must include both the expected active release and the
expected pointer revision. An A->B->A switch must invalidate an older
request that still expects A; checking only the active release is unsafe.
The injected/native adapter must preserve a durable revision/nonce-backed
compare-and-swap token enforced by the ordinary nonzero service behind the
gateway. The in-memory fake validates only this front-end contract; it does
not establish durable native service execution or authorization, and Program 0
performs no activation or authorization itself.
