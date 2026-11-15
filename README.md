# Skrynia Tools

Rust library and command-line tools for [Skrynia](https://github.com/ottojung/Skrynia). **The primary application workflow deploys a GitHub repository and a subdirectory**: check out an exact commit, run `make build` in a rootless Podman container, publish the `build/` tree as an immutable release, and atomically activate it. Existing releases can be rolled back.

**An application does not have to use an append-only log or any Skrynia storage.** Static deployment and optional data provisioning are separate. The Skrynia server remains an object store, not an application builder.

## Implemented

- `app new`, `app deploy`, `app inspect`, `app releases`, `app rollback` for static applications.
- Native Rust HTTP client for public GET, create-if-absent, replace, delete, byte-SHA256 CAS, paginated listing, child-grant creation and user-token issuance.
- `admin plan` and `admin create`: optional app data-grant provisioning, with private credential files.
- Optional `CasCell` and create-only `AppendLog` client protocols; `object` and `log` CLI operations.

**Not yet implemented:** a packaged browser npm/TypeScript facade, generic snapshot transactions, sealed-epoch garbage collection, task queues, and a standalone hosted deployment service. A GitHub Actions composite action now deploys over SSH using the native CLI on the static host. Initial browser WebAssembly bindings are provided as a separate build target; browser interoperability and packaging qualification are outstanding. These are separate proposals, not prerequisites for deploying applications.

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

The default builder is the tested `docker.io/library/rust:1.93-bookworm`; the builder image must contain `make` and the build dependencies of the app (use a custom Node/npm image for Node applications). The historical Skrynia builder image can be supplied explicitly if available. The CLI requires Git and **rootless Podman**. A default-branch commit is used and recorded if `--commit` is omitted; pin the exact SHA and builder image digest in production.

Configure your static server to serve `/srv/skrynia-apps/apps/` as `/apps/` using [the nginx example](examples/nginx-apps.conf). Releases, metadata and build workspaces remain outside that public path. Data grants are provisioned separately with `skrynia-tools admin create` and are never automatically exposed to a frontend.

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

CI enforces default rustfmt, native tests, Clippy, and Ivanka's Rust structural lint: at most **50 nonblank/noncomment lines per Rust file, 10 direct entries per source directory, and sibling-only imports**. The deployment suite exercises a real Git checkout with an isolated simulated builder; a separate manual qualification has exercised a real rootless Podman container and the current Skrynia HTTP server.

Skrynia Tools is Free Software under **GNU AGPLv3 or later**; see [LICENSE](LICENSE).
