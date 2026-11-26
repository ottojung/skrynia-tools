# Disposable nested-P2 callback integration (T6 interpreter selection)

This directory is a disposable test fixture, not a deployable artifact. It
exercises the T3/T6 P1 (`cb_guest::eval::P1CallbackEval` selecting
`p2::Interpreter`) under a compiled K4 kernel and T2's target P0 with a fake
bearer and temp directories only.

`nested_cb.py` verifies exact artifact SHA-256 pins, boots the kernel, stages
active/blob/appdata fixtures directly on the disposable host dirs, and invokes
four real WASM guests compiled from the included WAT:

- `echo.wat` — T6 interpreter roundtrip with no callback,
- `cb_roundtrip.wat` — one `CB_GET` plus `CB_PUT` drain of a real P1-owned
  appdata `AppGet` reply,
- `cb_failure.wat` — sticky callback failure and no-replay observation,
- `malformed_import.wat` — fail-closed validation (`skrynia.eval` import),
- `unknown_op.wat` — an unrecognised `p0.invoke` op returns `EINVAL (-3)`
  instead of being silently ignored.

`dumb_p0_http.py` is the unmodified disposable HTTP harness. Kernel, P0, P1
and workdir paths can be rebound with `SKRYNIA_KERNEL_WASM`, `SKRYNIA_P0_WASM`,
`SKRYNIA_P1_WASM` and `SKRYNIA_NESTED_WORKDIR`; all SHAs are re-pinned in
`nested_cb.py`. With T2's ops-5-to-10 P0 (`bb3b8afb`) and the size-optimized
P1 (`e9a65426`), the harness reports `T3_T6_NESTED_CB_PASS` (6/6). Artifacts
and scope are recorded in `docs/target-t3-native-service-port.md`. Nothing here
is production, a packaged P1 module, or Theorem Lab end-to-end acceptance.
