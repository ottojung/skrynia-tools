# Skrynia Tools contributor and agent contract

Read README, docs/architecture.md, docs/security.md, and the relevant protocol specification before changing code. Adopt the transferable principles of [Volodyslav's AGENTS.md](https://github.com/ottojung/volodyslav/blob/master/AGENTS.md), but this project is Rust, not JavaScript/JSDoc. Skrynia clients are independently untrusted: the non-adversarial frontend assumption does not transfer.

## Capabilities, types and security

- All side effects go through **injected capabilities**: HTTP transport, secret storage, filesystem, clock, randomness and diagnostics as applicable. Protocol state machines must be testable with deterministic fake capabilities, without ambient I/O.
- Distinguish Skrynia-enforced storage guarantees, cooperative protocol guarantees, and independently verified reader guarantees. Never claim server-side multi-object atomicity, private reads or enforcement of client-defined protocols.
- Treat wire data and independently written clients as hostile. Keep root and issuer tokens out of public objects, client bundles, URLs, Git history, normal CLI JSON and logs.
- Make invalid states difficult to represent. Use private constructors, Rust enums/newtypes, precise `Result` failures and boundary validation; avoid ambiguous Boolean switches, generic errors and unchecked casts. Define nominal-type invariants and explain exactly which constructors prove each property.
- Differentiate conflict, quota rejection, unauthorized access, invalid data and unknown mutation outcomes. Never silently replay a write after an uncertain network response.
- API docs and code comments explain living contracts, proof obligations and subtle failure modes. Never narrate obsolete code, prompts, workarounds or PR history in source comments.

## Ivanka structural Rust lint rules (mandatory)

- **At most 50 nonblank, noncomment lines in each .rs file**.
- **At most 10 direct entries** (files and directories) in any folder under `src`, `source_lints`, `tests`, `examples`, or `benches`.
- **Sibling-only internal imports**; no `super::super`, grandchild module traversals or nested path-module escapes.
- **Default rustfmt**, no formatting overrides: `cargo fmt --all -- --check`.
- The build-time `source_lints` and their self-tests enforce these rules; do not disable or weaken them. Refactor into meaningful small modules instead of compressing code to evade limits.

## Testing and workflow

Understand Skrynia's actual API and Cookbook; document authority and failure assumptions; implement behind injected capabilities; test concurrency, lost replies, crash points, malformed payloads and adversarial writers. Run `cargo fmt --all -- --check`, `cargo test --locked --all-targets`, `rustc --test source_lints/mod.rs -o /tmp/skrynia-lint-tests && /tmp/skrynia-lint-tests`, and `cargo clippy --locked --all-targets -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity`. Report checks not run. Make small conceptual commits and keep normative docs consistent with implementation. Do not weaken lint/type settings without direct approval.

Volodyslav convention for GitHub discussion posts: light metaphor or humor where fitting and end with `@ottojung`.

## Intent Records

`docs/intent-records/` records **current directly sourced user intent**, not agent-derived decisions. Use random stable opaque `$id-<16 decimal digits>`, descriptive title, establishment date, provenance and kind. Do not forge `source: @ottojung` from an agent prompt, issue or code artifact. Surface conflicting intents and remove withdrawn ones, letting Git carry their history. Derivative architecture belongs in the design documents.
