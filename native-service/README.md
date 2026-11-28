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

On the target ABI the guest uses explicit kernel root tokens: every registry
path is derived from `user/programs`, so writes, reads, listings and fsync
parents select the confined user root. Deliberate operator reads stay in the
operator namespace. Absence is only the kernel's `-8` ENOENT; `-2` EBADF, `-5`
EIO, `-4` ENOTSUP and unsafe-path failures remain hard errors, so a broken
descriptor or I/O fault is never silently treated as a missing object. P1's
`MutLock` is the P0-mediated shared userland lease: `1` acquired, `0` contended
and a negative code for a non-owner release.

ABI 3.1 (`skrynia_v3_1`) moves all host buffer arguments to `u64` offsets into
the calling root's exported non-shared linear memory. This compiled P1 guest
has no direct kernel imports: it reaches descriptors only through the
unchanged `p0.invoke` byte protocol, so its zero/one-callback `eval` semantics
and application authority are unaffected. The staged paths, code, random
bytes and bulk data live in P0's private memory; the lease and owner tags are
the only deliberate shared-RAM protocol and remain userland coordination.

A successful host test is not compiled guest acceptance. Use the pinned
disposable integration harness, independently verify source and artifact
identity, and test least-authority delegation, concurrent mutation, restart
and recovery before release.
