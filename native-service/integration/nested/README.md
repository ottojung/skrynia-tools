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
- `malformed_import.wat` — fail-closed validation (`skrynia.eval` import).

`dumb_p0_http.py` is the unmodified disposable HTTP harness. The pinned
artifacts and current blocker are recorded in
`docs/target-t3-native-service-port.md`. Nothing here is production, a
packaged P1 module, or Theorem Lab end-to-end acceptance.
