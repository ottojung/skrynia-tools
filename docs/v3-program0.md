# Optional Program 0 for Skrynia v3

Skrynia is **only the VM server** and does not bundle any Program 0.
This repository supplies **one optional** user-controlled WASM implementation
under `program0/`, compiled independently and installed by the operator.
An operator may instead supply any other conforming Program 0 module.

The VM loads a module from `SKRYNIA_ROOT_WASM` on every initialization or
restart. The module exports:

```text
skrynia_main(input_fd: i32, out_ptr: u32, out_cap: u32) -> i32
```

The first argument is a read-only, forward-only HTTP body descriptor, **not**
a WASM memory address. The module reads with the privileged
`skrynia.read(input_fd, dst_private_ptr, len)` host import until EOF and may
close the input descriptor. Attempted write/seek/truncate/sync fails. Program
0 has its own private linear memory, wholly separate from global RAM.
Evaluated child programs do **not** inherit Program 0 privileges.

## Build

Use Rust 1.93 and the `wasm32-unknown-unknown` Rust target:

```sh
cargo build --locked --release -p skrynia-program0 --target wasm32-unknown-unknown
```

The output is `target/wasm32-unknown-unknown/release/skrynia_program0.wasm`.
With Rust source installed but no target stdlib, the equivalent development
cross-build is `RUSTC_BOOTSTRAP=1 cargo -Z build-std=std,panic_abort build
--target wasm32-unknown-unknown --release -p skrynia-program0`.

Point a **disposable Skrynia v3** instance's `SKRYNIA_ROOT_WASM` at this file,
and provide a separate, confined data directory. A release must not point at
an existing v2 data tree until migration and rollback are validated.

The current example program implements a small, application-defined
register/set/invoke/lookup/revoke protocol in WASM. It is **not yet a complete
implementation of the historical v2 grant-based wire protocol**. That parity
belongs to this repository's compatibility facade and is a release gate.
No CLI is required or supported for Program 0.
