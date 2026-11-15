# Skrynia Tools

Rust tooling for [Skrynia](https://github.com/ottojung/Skrynia): **GitHub-source static-app deployment**, a reusable HTTP client, optional data protocols, and an administration/developer CLI.

**Static application deployment does not require any Skrynia data protocol.** Apps can choose arbitrary client data structures—or use no persistent storage at all. The primary deployment workflow preserves historical Skrynia behavior: GitHub repository + commit + monorepo subdirectory, run `make build` to produce `build/`, publish an immutable release, switch it atomically and allow rollback. Hosting happens outside the Skrynia server.

## Documentation

- [GitHub-source deployment](docs/deployment.md)
- [Architecture](docs/architecture.md)
- [Optional data grants](docs/app-provisioning.md)
- [CLI interface](docs/cli.md)
- [Protocol catalog](docs/protocols.md)
- [Threats and recovery](docs/security.md)
- [Implementation roadmap](docs/roadmap.md)
- [Contributor rules](AGENTS.md)

## Build quality

Every Rust source file has at most 50 nonblank/noncomment lines; each Rust source/lint directory at most 10 entries; imports are sibling-only. The build-time enforcement and tests are imported from [Ivanka](https://github.com/ottojung/ivanka). CI checks default rustfmt, cargo tests, structural lint self-tests, and Clippy.

Free Software under **AGPL-3.0-or-later**; see [LICENSE](LICENSE).
