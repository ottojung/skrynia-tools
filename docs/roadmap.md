# Skrynia Tools roadmap

## Current components

Native Rust/WASM clients, optional cooperative data protocols, a token-gated
root proxy example, ordinary native storage and code registry, browser
administration, trusted edge, and isolated outbound push sender.

## Release work

- Reproducibly build and test compatible kernel, root and ordinary guests,
  trusted edge, sender and app clients from exact revisions.
- Qualify the admin UI against actual sealed versions and revision CAS.
- Qualify exact-key wake delivery, recovery, authorization and resource bounds.
- Provide a repeatable Git-based build and explicit `publish`/`unpublish`
  publication path with web-based version management and constant-work code
  rollback. Never rely on direct static serving of private storage.
- Validate per-app migration, backup, data authority and stable service-worker
  origin independently of code publication.

The application and operator determine data semantics. Production cutover is
distinct from source integration and disposable acceptance.
