> **Native v3 boundary:** Administration is a nonzero application/service,
> not Program 0. The legacy v2 HTTP grant/storage endpoints are unsupported
> by the new POST-only root. See v3-program0.md.

# Skrynia Tools web administration

Grant administration is a static application published to Skrynia. The server
remains generic: it never executes application code or implements a new
privileged endpoint. All operations use existing Skrynia HTTP grant APIs.

## Bootstrap once

Obtain a **grant user token** whose scope covers the application data you
want to administer (for example, `apps/*`). It needs `delegate` to create
child grants and `create`/`modify` authority for the admin vault and
installation files. A grant issuer token or the Skrynia root issuer token
is **not** a valid value for `SKRYNIA_TOOLS_GRANT`.

For an installation on `vau.place`, provision the **separate browser origin**
`https://tools.a.vau.place` first. It requires working wildcard/SAN TLS and
nginx routing from the app hostname to `apps/tools/*`, plus a same-origin
`/_skrynia/` API reverse proxy. Unlike the direct object URL on `vau.place`,
the app-host origin must allow trusted JavaScript to execute.

~~~bash
export SKRYNIA_URL=https://vau.place/_skrynia
export SKRYNIA_TOOLS_GRANT='YOUR_SCOPED_GRANT_USER_TOKEN'
export SKRYNIA_TOOLS_ORIGIN=https://tools.a.vau.place
curl -fsSL https://raw.githubusercontent.com/ottojung/skrynia-tools/main/deploy.sh | bash
~~~

Any of the three variables not present is requested interactively from the
controlling terminal. The grant input is read without terminal echo; never
paste it into shared shell history. The bootstrap requires Bash, curl, tar,
and Node.js 22 or newer. It downloads Skrynia Tools source and publishes its
web assets directly over Skrynia HTTP using the supplied user token. No SSH,
privileged admin daemon, CLI executable, installation on the Skrynia host,
or extra API endpoint is required.

For a reviewed and reproducible installation, fetch `deploy.sh` at an exact
Git SHA and export `SKRYNIA_TOOLS_REF` to **that same SHA**. An already checked
source tree can be used with `SKRYNIA_TOOLS_SOURCE_DIR=/path/to/skrynia-tools`.
The optional `SKRYNIA_TOOLS_APP` defaults to `tools`; its name must match the
app-host routing.

On success the script prints the **admin URL and a random 256-bit access
secret**. Save that secret privately. It is not part of the URL and is never
sent to Skrynia. Losing it can make the vault irrecoverable, including issuer
credentials that cannot be issued again.

## What is stored

The bootstrap generates a random vault key under
`apps/tools/vault/<random>.json`. The vault stores the parent grant user token
and managed child issuer/user tokens **encrypted** with PBKDF2-SHA256
(310,000 iterations, random salt) and AES-256-GCM (random nonce). The random
access secret contains 256 bits of independent cryptographic randomness.
Encryption binds ciphertext to the vault key with authenticated associated
data. The public release's `config.json` contains only that vault key, not
the access secret, ciphertext plaintext, or any grant authority.

Grant and user actions in the browser require unlocking the vault.
The client then sends normal bearer authorization headers directly to the
same-origin `/_skrynia/` API; Skrynia enforces scope, operations and quota.
Nothing about the password is enforced by Skrynia itself: **this is client-side
credential encryption, not server-side login**.

The admin app exposes grant creation, imported issuer credentials,
user-token issuance, grant inspection (when the matching user token is held),
user revocation and grant subtree revocation. Credentials are displayed only
on explicit actions. The browser retains the access secret in memory while
unlocked, and does not persist it to localStorage or IndexedDB.

The UI is a trusted application. Anyone who can replace its JavaScript can
attempt to steal the access secret when the user unlocks it. Consequently,
the issuer and deployment grants for **the admin code** must be carefully
protected. The vault's encryption does not protect against malicious code
executing on the admin origin, a compromised device, or a compromised Skrynia
server/operator. Public `GET` is expected. The nginx app host must not log
capability-bearing paths and must set `Referrer-Policy: no-referrer`.

## Applications, versions and atomic activation

The web interface discovers apps by listing the parent grant's `apps/*`
namespace and recognizing release markers and active pointers.
It shows every `ready` release whose manifest is valid (and labels
malformed manifests as unavailable), including commit IDs and file counts.
An unfinalized partial upload is not a selectable version. This scan
uses the native Skrynia listing protocol with pagination, so scanning a
large namespace can be expensive. The list is not a cross-page snapshot:
concurrent uploads can alter the set during enumeration.

Choosing a version, whether older or newer, reads its ready marker and
release manifest, reads the current small `active.json` object, then
conditionally replaces that pointer with a fresh nonce using
`X-Skrynia-If-SHA256`. The change affects **one object only** and is
atomic under Skrynia's single-key CAS semantics. A concurrent activation
fails rather than silently overriding the other writer. No release
assets or application data are copied, rewritten or migrated.

The parent grant needs `list` authority over `apps/*` to discover
all apps, and `modify` authority for each app's `active.json`
pointer to activate releases. An app without an active pointer can
be activated by create-if-absent instead of CAS. This does not add
any special privilege beyond the supplied grant.

Versions are identified by random release IDs; their identifier order
is **not** a chronological order, and no timestamp is inferred.
The interface presents every available version for deliberate selection.
For the admin interface itself, it refuses switching to a code
version tied to a different encrypted vault key, preventing
accidental loss of access to its stored credentials.

## Mutation recovery

Skrynia offers single-object CAS but **not atomic multi-object grant
issuance + vault updates**. The browser saves an encrypted `pending` record
with byte-SHA-256 CAS *before* sending a new grant, user or revocation request.
It saves the returned issuer/user credential in the encrypted vault **before**
reporting success or displaying a user token.

An uncertain mutation response leaves `pending` in the vault and blocks
new actions. It is **not retried automatically**. If the server returned a
one-time credential but the subsequent vault write failed, the UI offers a
private emergency recovery download. A lost one-time issuer response cannot
be recovered through the existing grant API. The administrator must investigate
an unresolved operation and explicitly acknowledge the outcome before
performing another grant mutation. An uncertain encrypted-vault write also
blocks the current browser session until the vault is reloaded.

These recovery steps are client obligations, not server-enforced guarantees.
A committed orphan grant can still exist after a lost issuer response.
Revoking grants never erases stored objects.

## Deployment layout and tests

The bootstrap publishes `apps/tools/releases/ID/*`, a release manifest and
ready marker, a stable `apps/tools/index.html`, and a CAS-updated
`apps/tools/active.json`. The HTML assets use relative URLs so a gateway
can serve them from `https://tools.a.vau.place` without rewriting their code.
The vault object is **outside** the immutable code release. A UI-code rollback
does not copy, delete or reset credentials.

To test without production credentials, build a Rust Skrynia server and run:

~~~bash
SKRYNIA_TEST_SERVER_BIN=/path/to/skrynia-server \
SKRYNIA_TEST_BROWSER=1 bash tests/web-admin/live.sh
node tests/web-admin/faults.mjs
~~~

The tests issue a throwaway scoped parent grant on a disposable local server,
publish the site, exercise grant creation and revocation, and use Chromium to
verify password handling, locking and credential actions. Browser tests use a
loopback app-origin fixture; final qualification also requires testing the
real HTTPS hostname, certificate, nginx configuration and authorized grant.
