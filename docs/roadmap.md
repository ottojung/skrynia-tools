# Implementation sequence

The primary app deployment path is **repository URL + subdirectory → exact checkout → containerized `make build` → immutable release → atomic symlink activation**. Its lifecycle is independent of any Skrynia data protocol.

1. Native GitHub static deployment and rollback, with source/commit provenance and failed-build isolation.
2. Typed Skrynia HTTP operations and data grant administration. No data grant is needed for a static application.
3. Optional CAS cells and create-only append-only logs, with conflict/ambiguous-result tests.
4. Optional logical transactions, event sets and epoch-based log collection, subject to formalized safety and liveness assumptions.
5. Browser/Wasm compatibility, with the same Rust algorithms and a small JS bridge.
6. Further client-side coordination as individually justified protocols rather than mandatory app infrastructure.

Use deterministic unit tests, real Skrynia API tests where possible, and security tests around build containers, secret handling, root/symlink path escapes and partial deployment failures. The license is AGPL-3.0-or-later.
