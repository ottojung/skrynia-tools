# Native service scaffolding — NOT Program 0

This non-root Rust library preserves the earlier prototype of native
application storage, capability checks, registry, code history and activation
algorithms as source and tests. It is NOT a runnable Program 0: the privileged
root entry point has been removed, and the new Program 0 lives exclusively in
the separate program0/ crate.

The library is not an operator-ready evaluated guest program either.
The current guest ABI accepts only skrynia.invoke as an import and does not
allow direct file, RAM or HTTP access. Reusing this library inside a nonzero
guest therefore requires designing and verifying an explicit, least-authority
service boundary, not merely compiling this old privileged root code.

Do not include this library in the root WASM or deploy its previous private
binary protocol. The old service wire operations, including application
registration and version activation, are not part of the Program 0 contract.
Their tests verify implementation pieces for future migration, NOT that native
applications are ready to deploy.
