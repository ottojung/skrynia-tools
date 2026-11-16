# Skrynia Tools architecture

**Native Rust core, CLI and eventual WebAssembly bindings.** The Skrynia server remains a simple public byte-object store with scoped grants. It does **not** build or host applications. The Tools library implements reusable **optional** client data protocols independently of the code-artifact publishing workflow.

## Subsystems

1. **Native Skrynia client:** real GET, create-if-absent, replace, delete, SHA-256 conditional mutation, paginated listing and grant management. Typed failures distinguish conflict, unauthorized, quota and ambiguous network outcomes.
2. **Optional data protocols:** CAS cells, create-only append logs, event sets and logical snapshots. Each protocol lists its own exact authorities, trust assumptions and wire format; applications are never presumed to use any one of them.
3. **Native admin/developer CLI:** ordinary object operations; optional data-grant provisioning; GitHub subdirectory application-code deployment and rollback, running independently of the Skrynia server.
4. **Browser/native interoperability:** in future, compile the same Rust protocol algorithms to WebAssembly, with a thin JS adapter; do not reimplement protocol semantics in TypeScript.

## Side-effect boundaries

Keep source checkout, build isolation, local release publication, HTTP, credentials, clock and randomness behind narrow modules/traits where appropriate. Never pass operator credentials into app source build containers. Typed protocol state must be validated from untrusted bytes; a caller with broad write authority can bypass any library-side protocol.

## Hosting distinction

Skrynia object keys cannot contain slashes. The original GitHub-subdir deployment produced hierarchical `build/` files and was served from immutable release directories with atomic current-symlink switches. That **cannot** be transparently reproduced by merely storing individual assets under flat object keys. An administrator-controlled local release root holds the code. A static host can serve static assets from it; other runtime arrangements are separate. Skrynia stores application data if required. See [deployment](deployment.md).

## Definition of done

A feature is implemented only with real executable commands, validated tests and documented failure behavior. Proposed modules remain explicitly marked as such.

## Optional immutable snapshots

`cas::snapshot::SnapshotStore` offers a client-side multi-object logical snapshot without adding transactions to the Skrynia server. It stages immutable byte objects and a manifest, then CAS-replaces one head pointer; readers validate the pinned manifest and every blob. Old data and failed staging are retained, and no sweeping or deletion is automatic. This protocol is entirely optional and unrelated to serving/deploying an application.
