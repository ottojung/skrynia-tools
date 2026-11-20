# Skrynia v3 Program 0: two-FD streaming proxy only

The optional root in program0 is independently built and operator-supplied.
It imports only skrynia.run_program1(input_fd:i32, output_fd:i32)->i32 and
exports skrynia_main(input_fd:i32, output_fd:i32)->i32. It passes opaque
descriptors unchanged and returns the kernel result. It never parses,
buffers or routes requests, authorizes clients, stores user data, registers
programs, activates versions, or implements administration.

The server is configured with SKRYNIA_ROOT_WASM and the independent
SKRYNIA_PROGRAM1_WASM. The kernel prevalidates Program 1 at startup and
creates a separate VM per request. Input FD is read-only, output FD is
write-only; the child's import allowlist contains only bounded descriptor
operations, not disk, RAM or the privileged root API. Operators, not root
business logic, install/select Program 1.

This replaces the former buffered proxy that loaded programs/root.wasm
through a root disk import and copied the entire input/output into WASM
memory. The kernel's separate in-memory eval interface remains distinct.
Native streaming transfers the descriptors, uses bounded chunks, and does
not impose the former proxy's 16/8MiB whole-message buffers.

Build: cargo build --locked --release -p skrynia-program0 --target wasm32-unknown-unknown

Disposable integration test (against separately built kernel and proxy WASM):
SKRYNIA_V3_SERVER_BIN=/path/to/skrynia-server
SKRYNIA_V3_PROGRAM0_WASM=/path/to/skrynia_program0.wasm
cargo test --locked -p skrynia-tools --test v3_proxy -- --ignored

The real tests must verify exact opaque byte forwarding at 320 KiB and
4 MiB, child denied root filesystem, missing guest fail-closed and
resource bounds. No CLI, v2 HTTP GET/store compatibility, or application
semantics are part of Program 0.

The former registry, appdata authorization, ready code history and O(1)
activation are retained only in an unreleasable native-service library for
extracting into ordinary user programs. The isolated Program 1 currently
cannot perform persistent data operations without a separately reviewed,
least-authority capability API. All five app migrations and browser admin
remain blocked on that capability boundary and independently tested
data migration/rollback; no production installation is authorized.

## Separate stateful Program 1 example

The native-service/ crate is now a separately built user Program 1 example.
It exports its own skrynia_main two-FD handler and implements application
protocol/storage in Program 1, never Program 0. Its trusted operator must
configure SKRYNIA_PROGRAM1_DATA_DIR as a dedicated private directory, and
provision that directory's programs/operator.cap secret offline.

The kernel enforces the capability root, path safety and guest import
restrictions. The Program 1 service enforces application-level separate
read/write capabilities and CAS. It may evaluate unprivileged children
without giving them host disk rights. This distinction allows the dumb
Program 0 to remain exactly a descriptor forwarder.

The actual three-component integration suite is tests/v3_live/ (ignored
unless invoked explicitly with the real external binary and both root
and Program 1 WASM paths). It tests stored appdata, guest callbacks,
ready-version listing and release CAS/rollback across restarts.
No production data migration, rate limits or independent security audit
has been completed. This example is a draft service, not a bundled kernel
dependency.
