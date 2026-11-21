# Durable v2 issuer administration: compiled-WASM live validation

This record is a reproducible validation of the optional ordinary nonzero
v2 WASM provider's issuer/grant administration on the generic v3 kernel.
It is **not** a release qualification or a full v2 parity claim. The kernel
continues to know only the operator-supplied root Program 0; all v2 routes,
authority and state live in the unprivileged nonzero guest.

## Source identity

- Repository: `https://github.com/ottojung/skrynia-tools.git`
- Implementation base on this branch: `c4632edac3b244d9cb538091d54f549020455a3e`
  (`origin/agent/Jmf1xmEs6yST2sOvBNYUTN-v2-issuer-continue`, draft PR #46),
  stacked on PR #43 head `df1d03d26e74c84da4c7d62c9fcd2a4fbe9d8ddb`.
- Kernel: `https://github.com/ottojung/Skrynia.git` at
  `62ea225858f2a5d6a08cde6c9b1b9abbbbb0cb2e` (draft PR #112, stacked on
  generic callback-scope PR #111 `2f608ddaf5446c4e6b58f97af8dac4de378d3e5f`).

## Endpoints owned by the guest

| Method | Path | Authority |
| --- | --- | --- |
| POST | `/grants` | parent user bearer; `delegate` op; strict child narrowing |
| GET | `/grants/{id}` | exact grant user bearer |
| POST | `/grants/{id}/users` | exact issuer bearer of that grant |
| DELETE | `/grants/{id}/users/{userId}` | exact issuer bearer of that grant |
| DELETE | `/grants/{id}` | exact issuer bearer; cascades to descendants |

Routing is in `src/wire/v2compat/route/admin/dispatch.rs`; durable mutations
are in `v2-provider/src/durable/admin/`. Grant JSON is written atomically
under the kernel's request-owned mutation lock. The v2 compatibility
inventory defines no CAS header for admin routes; compare-and-swap remains
the `/store` object precondition (`X-Skrynia-If-SHA256`), implemented in
`v2-provider/src/durable/ops.rs` with the same lock and atomic staging.

## Artifacts and exact build recipe

Built from the implementation base above with Rust 1.93.0, the isolated
rust-src snapshot at `/workspace/skrynia-buildstd-sysroot`, and
`wasm-ld` 14.0.6:

```
export RUSTC=/workspace/skrynia-buildstd-sysroot/bin/rustc
export RUSTC_BOOTSTRAP=1
export CARGO_TARGET_DIR=<own-target-dir>
export CARGO_BUILD_JOBS=2
export RUSTFLAGS="-C linker=/gnu/store/2a6hs7mwkl9abl9bxmm4akq2akq9b9ip-lld-14.0.6/bin/wasm-ld -C link-arg=--export-table"
cargo build --locked -Z build-std=std,panic_abort \
  --target wasm32-unknown-unknown --release \
  -p skrynia-program0 -p skrynia-v2-provider --lib
```

- `skrynia_program0.wasm` SHA-256
  `961626d5765a84104b441b708af87a7afe712c97b3b2daa9c594c039bf52a819`
- `skrynia_v2_provider.wasm` SHA-256
  `ead70f3a3f55fadbb3a84756ce08bc404cfa03e5acebd23c17c855c3be5beb02`

Binary import audit: the root Program 0 imports only generic `skrynia.*`
syscalls (including `try_mutation_lock`/`release_mutation_lock`) and exports
`skrynia_main`, `skrynia_host_cb` and `__indirect_function_table`. The v2
guest imports exactly `skrynia.invoke`, `skrynia.read`, `skrynia.write` and
exports `skrynia_main` and `__indirect_function_table`; it has no direct
root filesystem, RAM, lock or HTTP import.

The kernel was built with `cargo build --locked -p skrynia-server` from the
commit above; its binary SHA-256 is
`d714f513a512a382c299ba33b1df3c07198d9236926991a6f30455145d1b6009`.
A newer local kernel that requires `SKRYNIA_OPERATOR_DIR` is **not**
compatible with this test fixture; use the pinned #112 build.

## Live differential evidence

The ignored real-stack tests in `tests/v3_proxy` were executed against a
compiled kernel, the compiled token-gated Program 0 above, and the compiled
v2 guest above over real loopback HTTP with a disposable data directory:

```
export SKRYNIA_V3_SERVER_BIN=<kernel-binary>
export SKRYNIA_V3_PROGRAM0_WASM=<skrynia_program0.wasm>
export SKRYNIA_V2_PROVIDER_WASM=<skrynia_v2_provider.wasm>
cargo test --locked -p skrynia-tools --test v3_proxy -- --ignored --test-threads=1
```

Results from this exact source and artifact set:

- Focused grant suite (`v2::positive::grants`, `--ignored`): **7 passed,
  0 failed**, including restart durability, concurrent creation and both
  revoked-user / revoked-grant negatives.
- Full real-stack `v3_proxy` suite (`--ignored`): **34 passed, 0 failed**.
  One stale fixture assertion that expected the old
  `POST /grants -> 501` stub was corrected to the implemented contract
  `401` when no bearer is supplied.
- Native v2 provider tests: **10 passed, 0 failed**; doc-tests not run
  (`rustdoc` binary is absent in this environment).
- Tools library tests: **51 passed, 0 failed**.
- `cargo fmt --all -- --check`: PASS. Structural source lints: **7/7 PASS**.
- Strict Clippy: PASS with `LD_LIBRARY_PATH` set so the build script can
  load `libgcc_s.so.1`; one non-denied `collapsible_if` style warning.
- v2 compatibility inventory harness `check`:
  `inventory_ok endpoints=17 corpus=12`.

Positive grant cases cover child grant creation, inspection, single-user
issuance, grant revocation, user revocation, restart durability and
concurrent creation. Negative cases cover wrong issuer/user capability,
widened scope, revoked token use after restart, stale content CAS, quota
rejection and unreadable objects. All such routes are v2 requests carried
as data inside the single authenticated v3 `POST /`; no kernel HTTP method
was added.

## Limits

Not full v2 parity: Web Push, efficient quota scans, crash/power-loss
recovery, large objects, HTTPS, migration, production data and five-app
acceptance remain separate gates. The root grant is operator-provisioned
JSON; only hashes of bearer tokens are stored. Tokens used by the tests are
throwaway values. No production write, deployment or canonical main change
is part of this record.
