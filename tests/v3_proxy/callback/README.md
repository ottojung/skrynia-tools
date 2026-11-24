# Compiled callback isolation

`real_scope.py` runs the real compiled Program0 and a synthetic ordinary
WASM guest against a disposable generic kernel. The guest uses only the
stream and callback ABI. The fixture verifies positive user-root reads,
operator-only verifier/code denial, absolute/traversal/symlink escape denial,
operator data unchanged after mutation attempts, idempotent missing-user
path deletion, a same-name user-root twin, rejected callback EVAL and wrong
bearer, and successful requests after a deliberate guest trap.

This is the root isolation boundary: the operator-selected ordinary service
receives the user root, not the operator root. Isolation between untrusted
application namespaces is separately mediated by that service and exercised
by the compiled P2 adversarial fixture. The upstream kernel's compiled
`callback::nested::nested_guest_cannot_steal_root_disk_fd_after_callback`
regression additionally verifies retained descriptor isolation across nested
callbacks. The source-level Program0 wire tests are supplemental guards.

All three compiled fixtures run in the `compiled-stack` CI job. The probe is
built directly from the checked-in Rust source using Rust 1.93.0 and
`wasm32-unknown-unknown`; it uses only synthetic credentials and private files
in a temporary directory. No production configuration or data is read.
