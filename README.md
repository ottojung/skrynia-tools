# Skrynia Tools

**Design-stage** Rust client protocols and a CLI for [Skrynia](https://github.com/ottojung/Skrynia). The repository currently contains an enforced Rust project scaffold and proposed designs; the protocols and CLI are **not yet implemented**.

Skrynia stays a small public object store. This project supplies reusable client-side abstractions, grant provisioning workflows, native Rust support, and later browser/WebAssembly integration.

## Design and engineering documents

- [Architecture](docs/architecture.md)
- [Application provisioning](docs/app-provisioning.md)
- [CLI design](docs/cli.md)
- [Protocol catalog](docs/protocols.md)
- [Security and recovery](docs/security.md)
- [Implementation roadmap](docs/roadmap.md)
- [Contributor rules](AGENTS.md)

## Structural Rust rules

Every Rust file is limited to 50 nonblank/noncomment code lines; each Rust source/lint directory to ten direct entries; imports to sibling modules. Default rustfmt, structural lint tests, native tests, and Clippy run in CI. The enforcement is based on [Ivanka's source lints](https://github.com/ottojung/ivanka/tree/main/source_lints).

Run `cargo fmt --all -- --check`, `cargo test --locked --all-targets`, and `cargo clippy --locked --all-targets -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity`.

Skrynia Tools is Free Software licensed under the **GNU Affero General Public License v3 or later (AGPL-3.0-or-later)**. See [LICENSE](LICENSE).
