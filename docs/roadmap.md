# Skrynia Tools roadmap

The primary deployment path is GitHub repository/subdirectory, exact
checkout, local or CI-runner build, hierarchical Skrynia HTTP upload,
immutable release-qualified assets, and single-key CAS activation.

## Implemented

- Native HTTP client, scoped grants, object operations and optional
  append log, CAS cell, event set, and immutable snapshots.
- Native application CLI: local Git-based builds, prebuilt-directory
  upload, release inventory, current version, manifests, comparisons,
  SHA-256 verification, and constant-work CAS rollback.
- Reusable HTTP-only GitHub composite action: build on the ephemeral
  runner, publish via a scoped Skrynia user token. No SSH, extra
  deployment server, remote filesystem or cutover variable.
- Real Skrynia server integration tests for upload, nested asset paths,
  browser entrypoint, rollback, history, quota and grant compatibility.
- Optional browser WebAssembly package and cross-browser protocol tests.

## Remaining work

- Separate immutable asset-create authority from pointer-modify
  authority in the GitHub Action to enforce historical immutability.
- Robust deployment reconciliation after a lost activation reply.
- Explicit retention and garbage-collection policy for abandoned
  versions and orphan uploads.
- Operational qualification of large uploads, HTTP timeouts, cache
  behavior, service workers, CSP, frontend routers and absolute assets.
- Publish the browser SDK to npm when separately approved.
- Extend isolated adversarial/failed-I/O integration coverage.

Only Skrynia's ordinary HTTP object operations are required to serve
and mutate a published app.
