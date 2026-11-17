# Skrynia Tools

Rust client protocols, developer CLI, and reusable GitHub deployment action
for [Skrynia](https://github.com/ottojung/Skrynia).

**Deployment is entirely over Skrynia HTTP.** No SSH connection, deployment
daemon, application hosting server, separate release filesystem, or
deployment-specific HTTP endpoint is needed. Build locally or in GitHub
Actions, upload original directory layouts with a scoped user token, and
atomically activate one release by CAS-replacing a small pointer.

## Application releases

~~~text
apps/hello/index.html
apps/hello/active.json
apps/hello/versions/<release-id>.ready
apps/hello/versions/<release-id>.json
apps/hello/releases/<release-id>/index.html
apps/hello/releases/<release-id>/assets/main.js
~~~

The stable index.html reads active.json and navigates to the chosen
immutable version, so relative HTML/CSS/JS URLs resolve naturally.
A fixed-size ready marker and active pointer make rollback **O(1) in
history size and artifact bytes**. No data migrations run during rollback.
Manifests record per-file SHA-256 digests for explicit verification.

Builds must use relative asset URLs. Host-specific absolute URLs,
service-worker scopes and frontend routers may need configuration;
Skrynia does not rewrite arbitrary client source.

## CLI examples

~~~sh
cargo build --locked --release
export SKRYNIA_TOKEN='RESTRICTED_USER_TOKEN'
skrynia-tools app deploy --url https://example.org/_skrynia --name hello \
  --repo https://github.com/OWNER/REPOSITORY --commit FULL_40_CHAR_SHA \
  --subdir apps/hello
skrynia-tools app deploy --url https://example.org/_skrynia --name hello \
  --directory ./apps/hello/build --commit FULL_40_CHAR_SHA
skrynia-tools app list --url https://example.org/_skrynia
skrynia-tools app versions --url https://example.org/_skrynia --name hello
skrynia-tools app current --url https://example.org/_skrynia --name hello
skrynia-tools app compare --url https://example.org/_skrynia --name hello \
  --from VERSION_A --to VERSION_B
skrynia-tools app verify --url https://example.org/_skrynia --name hello --release VERSION
skrynia-tools app rollback --url https://example.org/_skrynia --name hello --release VERSION
~~~

All app-management commands use HTTP, with a token taken from
SKRYNIA_TOKEN or the variable named by --token-env. No SSH keys,
remote shell accounts, or cutover variables are involved.

The reusable [GitHub Action](action/deploy/action.yml) builds on an
ephemeral GitHub runner and uploads with the same Rust CLI over HTTP.

The simplest publisher token requires create, modify and list for
apps/NAME/* with finite quota. It is intentionally not a root or
issuer token. This single-token design follows cooperative immutable
release conventions; for server-enforced immutability, use separate
create-only asset and mutable-pointer grants.

## Generic grant provisioning

The CLI also supports `admin grant` for any delegated scope, not just
application deployments. Use a parent grant's **user token** with
`--parent-grant TOKEN` or find the generated root grant with
`--parent-grant-is-root` and **the server's own `SKRYNIA_DATA_DIR`**:

~~~sh
export SKRYNIA_URL=https://vau.place/_skrynia
export SKRYNIA_DATA_DIR=/path/to/server-data
skrynia-tools admin grant --parent-grant-is-root --locations 'apps/*' \
  --max-bytes 500000000 --max-objects 15000
~~~

`SKRYNIA_URL` supplies the URL for every CLI command by default;
`--url` overrides it. The issuer and user token are saved in a
private credential file, never printed. See
[grant creation](docs/cli.md#generic-grant-creation).

## Other capabilities

- Native Rust HTTP client for public GET, create-only POST, PUT, DELETE,
  byte-SHA256 CAS, listings and grant management.
- Optional CasCell, AppendLog, EventSet, and CAS-backed SnapshotStore.
- App data-grant provisioning with private credential journaling.
- Browser WebAssembly builds and browser SDK qualification.

## Documentation

[Deployment](docs/deployment.md) · [CLI](docs/cli.md) ·
[Architecture](docs/architecture.md) · [Security](docs/security.md) ·
[Optional protocols](docs/protocols.md) · [Contributor rules](AGENTS.md).

Skrynia Tools is Free Software under GNU AGPLv3 or later.
