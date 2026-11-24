# Skrynia Tools roadmap

The primary deployment path is GitHub repository/subdirectory, exact
checkout, local or CI-runner build, hierarchical Skrynia HTTP upload,
immutable release-qualified assets, and single-key CAS activation.

## Implemented

- Native HTTP client, scoped grants, object operations and optional
  append log, CAS cell, event set, and immutable snapshots.
- Native web administration (`web-admin-v3/`): CLI-free static front-end,
  trusted gateway, native P1 opcodes 9-14, single revision CAS activation,
  explicit uncertain-outcome handling.
- Real Skrynia server integration tests for upload, nested asset paths,
  browser entrypoint, rollback, history, quota and grant compatibility.
- Optional browser WebAssembly package and cross-browser protocol tests.

## Remaining work

- Native transport port for generic CAS/log/event-set client bindings
  (owner-coordinated followup; reserved root protocol library paths not
  modified in this draft).
- Robust deployment reconciliation after a lost activation reply.
- Explicit retention and garbage-collection policy for abandoned
  versions and orphan uploads.
- Operational qualification of large uploads, HTTP timeouts, cache
  behavior, service workers, CSP, frontend routers and absolute assets.
- Publish the browser SDK to npm when separately approved.
- Extend isolated adversarial/failed-I/O integration coverage.

Only Skrynia's ordinary HTTP object operations are required to serve
and mutate a published app.
