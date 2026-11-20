# Skrynia v3 Program 0: transparent proxy only

Skrynia is a VM server. Its root Program 0 is supplied independently by
the operator, not by the kernel. The optional Program 0 in program0/ in this
repository is intentionally a **dumb proxy**. It does not interpret the
request body, select applications by ID, authorize clients, maintain a
registry, implement storage, issue capabilities, manage code versions,
perform rollbacks or operate an administration interface.

The proxy receives its root's two file descriptors through
skrynia_main(input_fd:i32, output_fd:i32)->i32. It loads the
operator-provisioned WASM guest from programs/root.wasm in the confined
Skrynia data directory, forwards the opaque input bytes as the child's
input buffer using the kernel's skrynia.eval import, and forwards exactly
the guest result bytes to the output FD. All application semantics belong
to the nonzero guest. The proxy does not register itself or fall back to any
protocol when that guest is missing.

The guest is not privileged. Its WASM may import only skrynia.invoke, and
the proxy passes no callback, so in this initial minimal configuration it
cannot obtain filesystem, shared RAM or other ambient root capabilities.
A guest that tries to import skrynia.open_disk is rejected. Operators
must provision root.wasm and, when appropriate, an independently designed
capability-limited service; those interfaces are not specified or
implemented by Program 0.

## Real limitation: not yet streaming through the child

The root accepts forward-only read and write FDs, but the kernel's current
child eval interface copies the request and result into the guest linear
memory. The proxy is therefore temporarily limited to 16 MiB input,
8 MiB child module, and 8 MiB output. It does not buffer or reinterpret
any binary protocol fields, but it still buffers the complete opaque payload.
A true zero-copy or backpressured streaming proxy requires a new kernel
capability-safe guest execution ABI with read-only input FD and write-only
output FD. That change must preserve guest privilege separation.

## Build and validation

Build the independent root WASM for wasm32-unknown-unknown:

    cargo build --locked --release -p skrynia-program0 --target wasm32-unknown-unknown

Supply the built skrynia_program0.wasm through SKRYNIA_ROOT_WASM and place
a trusted, independently built guest at programs/root.wasm within the
Skrynia data directory. No CLI administration or compatibility HTTP facade
exists.

The disposable real-kernel test suite is:

    SKRYNIA_V3_SERVER_BIN=/path/to/skrynia-server \
    SKRYNIA_V3_PROGRAM0_WASM=/path/to/skrynia_program0.wasm \
    cargo test --locked -p skrynia-tools --test v3_proxy -- --ignored

The tests cover byte-for-byte opaque forwarding, missing-guest fail-closed,
and guest import denial. They do not qualify persistent app services or
real application migration.

## Architecture and migration status

The prior root-owned application registry, storage, operator auth, and code
history have been moved out of Program 0 into the separate
native-service/ library, without a privileged root entry point. They remain
unshippable scaffolding until redesigned as ordinary, independently
sandboxed user programs. Their historical tests do not confer runtime
authority, nor do they establish that any app is migrated.

Native code history, immutable versions, authenticated activation/rollback,
capability issuance, data migrations, and the browser administration UI
must be implemented outside Program 0, in an authorized nonzero
application or separately reviewed service. A safe storage/callback
capability ABI and native streaming guest interface are outstanding
kernel-plus-userland design tasks.

The old direct Program 0 wire and on-disk formats are deliberately not
forward-compatible. Do not silently interpret them in the proxy; export
and migrate data through an explicitly trusted operator process. No
production cutover has been made.
