# Trusted Program 1 service scaffold

This crate adapts `skrynia-native-service` protocol and state algorithms
for an operator-installed trusted Program 1 guest behind the dumb Program 0
proxy.

It provides:
- A reusable Rust dispatch API over injected `Disk`, `MutLock`, `Random`,
  and `Eval` capabilities (`service::open`, `service::dispatch`).
- A wasm32 guest entrypoint that currently fails closed because the
  kernel does not yet provide the required scoped trusted-capability ABI.

Durable storage, authenticated operator/admin persistence, and real guest
evaluation are **not** working end-to-end without the kernel extension
described in `../docs/v3-trusted-program1-abi.md`. No blanket disk/RAM is
exposed to untrusted application Wasm. No v2 HTTP/CLI compatibility is
implemented. No production cutover.
