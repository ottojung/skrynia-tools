# Nested-callback LEAK→SAFE isolation regression

The compiled-artifact adversarial suite (`adversarial.rs`, `adversarial_more.rs`,
`adversarial_scope.rs`, `nested.rs`) existed at source commit `b1a8052d` under
this path. Every test was `#[ignore]`d with the reason "requires compiled
Program0 and callback-safe generic kernel", so it never ran in CI, and the
suite was later removed. Acceptance #2 requires a non-vacuous nested callback
capability-isolation regression that CI actually runs.

The authoritative regression is now the **compiled kernel test**, not a host
stub:

- `rust-server/tests/http/callback/nested.rs::nested_guest_cannot_steal_root_disk_fd_after_callback`
  in the pinned upstream Skrynia checkout. It boots the real generic kernel
  with a root WASM that tries to steal a retained root disk FD through a
  nested callback and asserts the response is `SAFE`.
- `.github/workflows/native-components.yml` (`compiled-stack` job) pins the
  kernel at `6e750f6a737a744495f98a84830b53b44186f38f` and runs
  `cargo test --locked -p libskrynia --test http callback::nested::nested_guest_cannot_steal_root_disk_fd_after_callback -- --exact`
  after building the kernel. (`skrynia-server` is the binary target; the
  Cargo package at this pin is `libskrynia`.)

The source-level unit test
`program0/src/proxy/host/wire_test.rs::leak_safe_nested_callback_isolation`
is retained as a cheap **supplemental** guard: it fails if the callback's EVAL
denial or path hygiene is removed. It is not a substitute for the compiled
kernel regression and is not presented as such.

No kernel is built locally for this task; the compiled regression runs only in
the CI job from the pinned upstream source.
