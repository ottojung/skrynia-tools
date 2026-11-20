# Skrynia v3 — dumb, streaming proxy root (Program 0)

The optional Program 0 WASM only forwards the two root descriptors to an
independently operator-provisioned Program 1 and propagates its return code.
It does not select code, buffer or interpret messages, perform auth, maintain
storage or grants, manage releases, or implement application callbacks.
Its one import is skrynia.run_program1(input_fd, output_fd).

Configure the kernel with SKRYNIA_ROOT_WASM (this independent proxy) and
SKRYNIA_PROGRAM1_WASM (the operator-selected separate user program).
The kernel transfers both forward-only HTTP descriptors to a fresh isolated
Program 1 VM, with separate resource budget. No root disk, grant or RAM
capabilities are given to the child. Missing/invalid Program 1 fails closed.

The former persistent application registry and admin algorithms are preserved
only as unreleasable native-service library code. Stateful user programs
need a separate least-authority storage/CAS capability interface, not
application semantics hidden inside Program 0. No production cutover.
