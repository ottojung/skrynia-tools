# Implementation sequence

The primary app deployment path is **repository URL + subdirectory → exact checkout → containerized `make build` → immutable release → atomic symlink activation**. Its lifecycle is independent of any Skrynia data protocol.

1. **Implemented:** native GitHub static deployment and rollback with provenance, rootless Podman builds and failed-build isolation; SSH-based GitHub Actions deployment is implemented with a transport smoke test. Live remote-host qualification remains.
2. **Partially implemented:** native Skrynia HTTP operations, child grants, issued user tokens and optional data grants. A complete credential lifecycle, more complete credentials lifecycle and recovery remain future work. Static apps need no grant.
3. **Initial implementation:** byte-oriented CAS cells and create-only append-only logs, with unit tests and real-Skrynia smoke tests. Complete fault-injection and model-based testing before production qualification.
4. Optional logical transactions, event sets and epoch-based log collection, subject to formalized safety and liveness assumptions.
5. **Initial implementation:** browser/Wasm HTTP operations and append-only log bindings, with shared native/Wasm log byte-format and key semantics. Complete Wasm target compilation, generated JS package, browser integration tests and JavaScript API refinements.
6. Further client-side coordination as individually justified protocols rather than mandatory app infrastructure.

Use deterministic unit tests, real Skrynia API tests where possible, and security tests around build containers, secret handling, root/symlink path escapes and partial deployment failures. The license is AGPL-3.0-or-later.
