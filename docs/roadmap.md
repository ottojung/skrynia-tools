# Implementation sequence

The primary app deployment path is **repository URL + subdirectory → exact checkout → containerized `make build` → immutable release → atomic symlink activation**. Its lifecycle is independent of any Skrynia data protocol.

1. **Implemented:** native GitHub static deployment, application inventory, historical release inspection, and constant-work rollback and on-demand SHA-256 artifact verification with separate build/activation locks, provenance, rootless Podman builds and failed-build isolation; SSH-based GitHub Actions deployment is tested against real isolated OpenSSH (including rejected host keys and unauthorized roots); rootless Podman and remote CLI version-management/rollback are qualified in local SSH integration tests. Independently operated production-host qualification remains.
2. **Partially implemented:** native Skrynia HTTP operations, child grants, issued user tokens and optional data grants. Administrative journaling, inspection and guarded user-token recovery are implemented. Grant-create issuer recovery, explicit grant/user-token revocation and full lifecycle testing remain future work. Static apps need no grant.
3. **Initial implementation:** byte-oriented CAS cells and create-only append-only logs, with unit tests and real-Skrynia smoke tests. Complete fault-injection and model-based testing before production qualification.
4. Optional logical transactions, event sets and epoch-based log collection, subject to formalized safety and liveness assumptions.
5. **Initial implementation:** browser/Wasm HTTP operations and append-only log bindings, with shared native/Wasm log byte-format and key semantics. Wasm target compilation, generated JavaScript bindings, headless Chrome integration and npm tarball assembly are implemented in CI. Independent Firefox/WebKit integration, public registry release and optional ergonomic TypeScript APIs remain.
6. Further client-side coordination as individually justified protocols rather than mandatory app infrastructure.

Use deterministic unit tests, real Skrynia API tests where possible, and security tests around build containers, secret handling, root/symlink path escapes and partial deployment failures. The license is AGPL-3.0-or-later.
