# One-time native-v3 trusted-host admin bootstrap

This is a one-time operator bootstrap, not a CLI. It publishes the v3 web
admin assets as immutable static files through the already-provisioned native
publication path and prints a stable link plus a freshly generated access
password. It has no subcommands and performs no general deployment actions.

## Owner intent / scope

Direct user messages 2026-11-17 19:53:50 and 20:38:44 UTC retain a one-time
trusted-host bootstrap while ruling out an administrative CLI. Commit
f92fbea5 removed only obsolete LEGACY v2 HTTP deploy routes and the v2 grant
form; it did not withdraw the bootstrap. This implementation is native v3: no
`/store`, no v2 grant, no HTTP deployment facade, no direct nginx writes, and
no kernel-side bootstrap.

## Operator prerequisites (not created by this script)

- A trusted HTTPS static origin for the admin app, with the same-origin
  gateway already provisioned (the gateway holds the server-only Program 0
  token and frames private POST bytes server-side).
- An operator-owned **private artifact root** that already exists as a real
  directory (not a symlink or alias), owned by the invoking uid and private
  (`0700` or stricter). It must be disjoint from the nginx document root and
  from the publication mailbox, both of which must also be configured as
  existing, non-aliased directories. The installer canonicalizes paths,
  rejects nesting/symlink aliases, and rejects exact same-directory bind
  mounts via dev/ino. Distinct bind-mount aliases cannot be detected portably
  from userspace; operators must not supply aliased mount paths. The installer
  does not create the root.
- A compiled publication root WASM and reference publish provider, plus a
  native publication helper that accepts the JSON protocol below and publishes
  through the root/provider (never by writing nginx files directly).
- Two scoped native authority files: a 32-byte lowercase-hex read capability
  and a 32-byte lowercase-hex admin capability, plus an optional JSON map of
  app name to scoped admin capability. These are native service caps, not a v2
  grant and not Program 0/root/operator/sender secrets.

## Inputs (environment or interactive /dev/tty prompts)

`SKRYNIA_ARTIFACT_ROOT`, `SKRYNIA_NGINX_ROOT`,
`SKRYNIA_PUBLICATION_MAILBOX`, `SKRYNIA_ORIGIN_URL`,
`SKRYNIA_NATIVE_READ_CAP_FILE`, `SKRYNIA_NATIVE_ADMIN_CAP_FILE`,
`SKRYNIA_PUBLISH_HELPER`, `SKRYNIA_VERIFY_HELPER`; optional
`SKRYNIA_NATIVE_ADMIN_CAPS_FILE`, `SKRYNIA_NATIVE_ENDPOINT`
(default `/_skrynia/`) and `SKRYNIA_ADMIN_APP` (default `admin-v3`).
Optional `SKRYNIA_ALLOW_NONTEST_ORIGIN=1` is the explicit operator-review
escape hatch for a real (non-`*.test`) production origin; by default such an
origin is rejected. The compiled end-to-end fixture sets it only for its
disposable loopback origin. `SKRYNIA_ALLOW_LOOPBACK_HTTP=1` is the matching
disposable-only exception to the HTTPS-origin rule.

The exact bootstrap bundle identity is `SKRYNIA_BOOTSTRAP_BUNDLE_ID`, the
SHA-256 of `bootstrap/SHA256SUMS`. `deploy.sh` computes/verifies that digest
from the out-of-band anchor (`SKRYNIA_BOOTSTRAP_SOURCE_SHA256`, or the already
checksum-verified piped bundle) and passes it to the installer; the installer
records it in the private `.installed` marker and publishes it in
`manifest.json`/helper payloads as `bundleId`. `SHA256SUMS` does not list
itself, so the anchor is out-of-band, not self-referential. This identifies
the exact bundle bytes only; it is **not** the running kernel/tools stack
identity (exact artifact pins are qualified by exact-stack CI separately).
The reviewed tools main baseline for this change is commit `91f9845b...` (the
topic parent), not the executed bundle identity.

## Pinned distribution

Two supported one-time entry modes, both requiring immutable reviewed source
and out-of-band checksum verification (no self-referential hash, no mutable
`main` fetch):

1. **Piped, one-time** (owner-requested form): pipe an immutable reviewed
   `deploy.sh` to bash **only** with out-of-band checksums. `deploy.sh` detects
   pipe mode, requires `SKRYNIA_BOOTSTRAP_SOURCE_REF` (immutable 40-hex
   reviewed commit) and `SKRYNIA_BOOTSTRAP_BUNDLE_SHA256` (out-of-band SHA-256
   of that commit tarball), downloads the immutable tarball, verifies checksum,
   extracts, and executes the local reviewed installer. No mutable branch is
   fetched. Example shape:
   `curl -fsSL https://raw.githubusercontent.com/ottojung/skrynia-tools/<40hex>/bootstrap/deploy.sh | SKRYNIA_BOOTSTRAP_SOURCE_REF=<40hex> SKRYNIA_BOOTSTRAP_BUNDLE_SHA256=<sha256> bash`
   (The operator must independently obtain both hashes from reviewed evidence.)
2. **Trusted local checkout/download:** obtain the reviewed source from an
   immutable/trusted git checkout or release artifact, verify the out-of-band
   SHA-256 of `bootstrap/SHA256SUMS` via `SKRYNIA_BOOTSTRAP_SOURCE_SHA256`,
   then run the local file. The SHA256SUMS file binds all listed bootstrap
   assets; it is not self-authenticating.

The installer itself performs no network fetch except the explicit piped-mode
immutable-bundle bootstrap described above.

## Helper protocol

`install.mjs` spawns `SKRYNIA_PUBLISH_HELPER` and then
`SKRYNIA_VERIFY_HELPER`, each with a single JSON object on stdin:

```json
{"action":"publish|verify|unpublish","app":"admin-v3","release":"<sha256>",
 "bundleId":"<sha256-of-SHA256SUMS>","origin":"https://...","endpoint":"https://.../_skrynia/",
 "files":[{"rel":"apps/admin-v3/<release>/unlock.html","abs":"/private/...","sha256":"<hex>"}],
 "stable":[{"rel":"apps/admin-v3/unlock.html","abs":"/private/...","sha256":"<hex>"}]}
```

The shipped helper is `bootstrap/native-publish.mjs`: it reuses the native
publication-root wire semantics (token-authenticated `POST /` with op 1,
u32 path length, exact UTF-8 rel path, bounded 4-byte reply) against the
operator's already-running compiled kernel/publication root and reference
provider (`SKRYNIA_PUBLICATION_URL`, `SKRYNIA_PUBLICATION_TOKEN_FILE`). It
never writes nginx files. For `verify` it independently GETs every served
file via `SKRYNIA_PUBLIC_BASE_URL`, hashes the bytes, and returns structured
JSON `{"ok":true,"release":"...","files":[{"rel":"...","status":200,"sha256":"..."}]}`.
The installer compares that roster field-by-field against its own expected
hashes (exact distinct roster, rejects duplicates/missing, binds to configured
origin/base); an arbitrary stdout substring is never treated as success.
On definitive publish+verify failure, it unpublishes through the same native op
2 before reporting failure. On unknown publish outcome or ambiguous verify, it
**does not** blindly unpublish; it fails closed and requires operator
reconciliation. Helper stderr is withheld from installer errors because an
operator token must never reach logs. Private capability files remain 0600 and
are never published; published static assets are 0644.

The stable entry remains `${origin}/apps/admin-v3/unlock.html`; the immutable
release is addressed only in the verified file roster and manifest. After
verified publication, the password is written to a private 0600 recovery file
under the artifact root (`password.<release>.txt`) and also printed once to
stdout; keep the recovery file private and delete/rotate per operator policy.

## Password and vault boundary

The installer generates a 32-byte random password (base64url, 43 characters).
It encrypts a vault with `web-admin-v3/crypto.js` (PBKDF2-SHA256, AES-256-GCM,
310000 rounds, context `browser-admin-authority`). Vault contents are
whitelist-only: `schema`, `endpoint`, `readCap`, `adminCap`, `adminCaps`.
No Program 0/root, operator, sender or other server-only token, legacy grant,
or plaintext secret is ever placed in the vault, any public asset, URL, log or
manifest. The encrypted vault is published as ciphertext and is served
publicly; only the unlock page decrypts it in the browser after the operator
enters the password, then installs `window.__skryniaNativeSession` and
navigates to the admin app. Without successful unlock the shipped UI remains
fail-closed and no native session capability exists.

The password is printed once on stdout after served-byte verification **and**
persisted as plaintext in the private `0600` recovery file
`password.<release>.txt` under the artifact root (created only after successful
publication and verification). That file is never published, never placed in
the vault/manifest/URLs, and stays private under operator policy; rotate or
delete it per that policy. Losing both the printed value and the recovery file
requires a new one-time install with a fresh release; existing published
releases are immutable.

## Disposable qualification

`bootstrap/tests/installer.mjs` covers refusal, vault scoping, password
strength, the private recovery file written after success, artifact-root
prerequisites and exact roster checks (both the source `SHA256SUMS` and the
published `manifest.json`). `bootstrap/tests/chromium_bootstrap.mjs` drives the
real unlock page in Chromium. The compiled publication plus Chromium end-to-end
fixture is documented in `bootstrap/tests/PUBLICATION_E2E.md`.
