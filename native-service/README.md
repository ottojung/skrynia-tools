# Native stateful user service — ordinary Program 1

This crate preserves the application storage, per-object read/write capability
checks, immutable code releases and revisioned CAS activation algorithms as
an independently compiled **ordinary Program 1**, not as Program 0.

It exports the two-FD streaming user entrypoint skrynia_main(input_fd,output_fd)
from src/guest/ and implements its **own** binary application protocol.
Program 0 remains a tiny opaque FD forwarder and does not import this crate.

The kernel must be launched with:
- SKRYNIA_ROOT_WASM: the separately compiled dumb proxy WASM
- SKRYNIA_PROGRAM1_WASM: this crate's wasm32-unknown-unknown release artifact
- SKRYNIA_DATA_DIR: the separate root VM's data directory
- SKRYNIA_PROGRAM1_DATA_DIR: a dedicated private scope granting this Program 1
  generic file operations, atomic rename/fsync, secure randomness, request
  mutation lock and nested sandboxed evaluation

This is a **trusted operator-installed Program 1**. It has all rights within
its deliberately limited directory, and itself enforces client capability
permissions for appdata and code deployment. Its evaluated application
children have no ambient filesystem or root RAM imports and can communicate
only through explicit callback mediation. The kernel rejects unauthorized
path traversal, symlinks and imports.

A dedicated, private 32-byte operator installation capability must be
provisioned at the scoped programs/operator.cap before the server starts,
with mode 0600. The service rejects obsolete unauthenticated registration.
Do not put this secret in a static web bundle, logs, code, or user URL.

The real-kernel three-artifact native test suite exercises registration,
appdata read/write authority, nested callback isolation, code history,
ready-version CAS activation and rollback, restart persistence and denied
operator credentials. See tests/v3_live/. This is NOT yet production-ready:
quotas, rate limits, strong time/space bounds, independent security audit,
migration of legacy data, and application/browser admin E2E remain open.
