# Disposable compiled P2 callback acceptance

This fixture uses the current P1 interpreter and the exact kernel, P0 and P1
artifacts supplied by the build. It registers two real Rust-compiled WASM
guests, writes and reads its own appdata via complete P1-owned callback
frames, checks the persisted value with the ordinary service, and exercises
incomplete frames, sticky malformed-callback errors, unknown operations,
wrong import types, request recovery and bearer denial.

Build probe.rs and invalid.rs with Rust 1.93 and the wasm32-unknown-unknown
target (cdylib, opt-level=s, panic=abort, strip=symbols). Pass the paths and
SHA-256 of all five artifacts to nested_cb.py; see the compiled-stack CI
workflow for the exact commands. Each run creates a disposable directory
and uses fake credentials only. No operator-root fixture data is pre-seeded
except the verifier and root WASM required to start the service.
