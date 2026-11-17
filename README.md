# Skrynia Tools

Browser-based administration, CI application deployment Actions, and
reusable client protocols for [Skrynia](https://github.com/ottojung/Skrynia).

**There is no command-line application.** Administration happens in the
web interface. A one-time `deploy.sh` bootstrap publishes that
interface; CI publishes apps via GitHub Actions without a CLI executable.

## Web administration

The admin site is itself hosted by Skrynia on its own browser origin.
It stores grant/user capabilities in an encrypted Skrynia object
protected by a random 256-bit access secret.

It can administer delegated grants, list every app and its ready release
history, and atomically activate any older or newer retained version by
SHA-256 CAS on the single `active.json` object. No assets or application
data are migrated during a code rollback.

The one-time bootstrap uses environment values or interactive prompts:

~~~sh
export SKRYNIA_URL=https://vau.place/_skrynia
export SKRYNIA_TOOLS_GRANT='SCOPED_GRANT_USER_TOKEN'
export SKRYNIA_TOOLS_ORIGIN=https://tools.a.vau.place
curl -fsSL https://raw.githubusercontent.com/ottojung/skrynia-tools/main/deploy.sh | bash
~~~

Use a reviewed pinned commit for both the downloaded installer and
`SKRYNIA_TOOLS_REF`. It prints the URL and random access secret.
See [web administration](docs/web-admin.md) for details and security.

## Versioned application releases

~~~text
apps/hello/index.html
apps/hello/active.json
apps/hello/versions/<release-id>.json
apps/hello/versions/<release-id>.ready
apps/hello/releases/<release-id>/index.html
apps/hello/releases/<release-id>/assets/main.js
~~~

The stable entrypoint selects the release under the app's own browser
origin, so relative assets continue to resolve within its versioned
directory. The GitHub [deployment Action](action/deploy/action.yml)
builds and publishes using Node HTTP modules; the
[stable PWA Action](action/publish-stable/action.yml) publishes
`sw.js`, `manifest.webmanifest` and `icon.svg` separately.
Neither provides a public CLI.

## Libraries

The repository also provides a reusable Rust HTTP client, optional CAS,
log, event-set and snapshot protocols, and a WebAssembly browser SDK.
These client protocols do not require a privileged server or particular
application data model.

[Deployment](docs/deployment.md) ·
[Web administration](docs/web-admin.md) ·
[Architecture](docs/architecture.md) ·
[Security](docs/security.md) ·
[Optional protocols](docs/protocols.md) ·
[Contributor rules](AGENTS.md).

Free Software under GNU AGPLv3 or later.
