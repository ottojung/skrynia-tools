# Skrynia Tools

Rust library and command-line tools for [Skrynia](https://github.com/ottojung/Skrynia). **The primary application workflow deploys a GitHub repository and a subdirectory**: check out an exact commit, run `make build` in a rootless Podman container, publish the `build/` tree as an immutable release, and atomically activate it. Existing releases can be rolled back in **O(1) work and O(1) extra memory** relative to release history and artifact size.

**An application does not have to use an append-only log or any Skrynia storage.** Code deployment and optional data provisioning are separate. The release manager stores arbitrary regular build artifacts without requiring any HTML entry point or making assumptions about application data. It preserves normalized executable permissions and records a versioned SHA-256 digest for on-demand verification. The Skrynia server remains an object store, not an application builder.

## Implemented

- Application CLI: `app new`, `deploy`, `list`, `doctor`, `current`, `versions`, `show`, `compare`, `verify`, `rollback` and `activate`, with JSON and human-readable views.
- Native Rust HTTP client for public GET, create-if-absent, replace, delete, byte-SHA256 CAS, paginated listing, child-grant creation and user-token issuance.
- `admin plan` and `admin create`: optional app data-grant provisioning, with private credential files.
- Optional `CasCell` and create-only `AppendLog` client protocols; `object` and `log` CLI operations.

**Not yet implemented:** public npm registry publication or a custom TypeScript facade, generic snapshot transactions, sealed-epoch garbage collection, task queues, and a standalone hosted deployment service. A GitHub Actions composite action now deploys over SSH using the native CLI on the static host. Browser WebAssembly bindings, generated TypeScript declarations and an installable npm `.tgz` are built and tested in CI. Public npm publication and independent browser compatibility qualification remain outstanding. These are separate proposals, not prerequisites for deploying applications.

## Build and create an application

```sh
cargo build --locked --release
target/release/skrynia-tools app new --name hello --directory ./hello
```

`app new` creates a simple Makefile-based static website without tokens or storage assumptions. To deploy an app from a GitHub monorepo on your static hosting machine:

```sh
target/release/skrynia-tools app deploy \
  --name hello \
  --repo https://github.com/OWNER/REPOSITORY \
  --subdir apps/hello \
  --commit FULL_40_CHARACTER_GIT_SHA \
  --apps-dir /srv/skrynia-apps \
  --builder-image docker.io/library/rust:1.93-bookworm
```

Set `SKRYNIA_APPS_DIR=/srv/skrynia-apps` in your shell to omit repeated `--apps-dir` flags when managing releases.

Manage historical versions without rebuilding or touching data:

```sh
skrynia-tools app list --apps-dir /srv/skrynia-apps
skrynia-tools app versions --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app show --name hello --release RELEASE_ID --apps-dir /srv/skrynia-apps
skrynia-tools app verify --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app rollback --name hello --release RELEASE_ID --apps-dir /srv/skrynia-apps
```

The default builder is the tested `docker.io/library/rust:1.93-bookworm`; the builder image must contain `make` and the build dependencies of the app (use a custom Node/npm image for Node applications). The historical Skrynia builder image can be supplied explicitly if available. The CLI requires Git and **rootless Podman**. A default-branch commit is used and recorded if `--commit` is omitted; pin the exact SHA and builder image digest in production.

For static websites, configure your static server to serve `/srv/skrynia-apps/apps/` as `/apps/` using [the nginx example](examples/nginx-apps.conf). Releases, metadata and build workspaces remain outside that public path. Data grants are provisioned separately with `skrynia-tools admin create` and are never automatically exposed to a frontend.

## Remote CLI deployment and rollback

All application release operations also work over **pinned-host OpenSSH**, with no privileged Skrynia server endpoint:

```sh
export SKRYNIA_APPS_DIR=/srv/skrynia-apps  # path on the hosting machine
skrynia-tools app versions --name hello \
  --ssh-host deploy.example.org --ssh-user deployer \
  --ssh-key ~/.ssh/skrynia-deploy --ssh-known-hosts ~/.ssh/skrynia-hosts
skrynia-tools app rollback --name hello --release RELEASE_ID \
  --ssh-host deploy.example.org --ssh-user deployer \
  --ssh-key ~/.ssh/skrynia-deploy --ssh-known-hosts ~/.ssh/skrynia-hosts
```

Use the same SSH options on `app deploy` to build a GitHub repository subdirectory on the hosting machine. A dedicated SSH key may be restricted to the root-scoped `app rpc --spec-stdin true` command; JSON requests never require application data or migration protocols. See [remote setup](docs/deployment.md#remote-cli-release-management).

## GitHub Actions deployment

Use [the SSH deployment action](action/deploy/action.yml) with a dedicated deploy account, a pinned host key and an SSH private key stored as a GitHub Actions secret. It invokes the same CLI on the static host, transmitting deployment parameters as JSON over SSH without putting secrets into URLs or client builds. See [the full workflow example](docs/deployment.md#github-actions-deployment).

## Documentation

- [Deployment workflow and historical compatibility](docs/deployment.md)
- [CLI reference](docs/cli.md)
- [Architecture](docs/architecture.md)
- [WebAssembly browser SDK](docs/browser.md)
- [Optional application data grants](docs/app-provisioning.md)
- [Protocol catalog and guarantees](docs/protocols.md)
- [Security boundaries and recovery](docs/security.md)
- [Implementation roadmap](docs/roadmap.md)
- [Contributor and Rust source rules](AGENTS.md)

## Quality and licensing

CI enforces default rustfmt, native tests, Clippy, and Ivanka's Rust structural lint: at most **50 nonblank/noncomment lines per Rust file, 10 direct entries per source directory, and sibling-only imports**. The deployment suite exercises real Git checkout and OpenSSH transport with isolated build fixtures; manual qualification has exercised a real rootless Podman container over SSH and the current Skrynia HTTP server. Production-host qualification is still independent.

Skrynia Tools is Free Software under **GNU AGPLv3 or later**; see [LICENSE](LICENSE).
