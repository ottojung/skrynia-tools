# Ordinary native service

This crate implements an ordinary nonzero WASM service. It owns application
data capabilities, immutable sealed code identity, revision CAS activation,
mediated child execution, and exact-key Web Push state. The root proxy is
separate and does not understand these operations.

Build the guest with feature `cb-guest` for `wasm32-unknown-unknown`.
It accesses resources through the parent-mediated guest ABI. Host-side
interfaces enable deterministic tests; they do not authorize deploying this
crate as the root program. See `src/protocol/wire/` for operation definitions
and `integration/` for compiled integration tests.

Application data is addressed by read capability and key; the write capability
hashes to the corresponding read capability. Code registration and
administration have their own authority. Exact-key subscriptions and durable
change hints are consumed by an isolated outbound sender, not by the kernel.

A successful host test is not compiled guest acceptance. Use the pinned
disposable integration harness, independently verify source and artifact
identity, and test least-authority delegation, concurrent mutation, restart
and recovery before release.
