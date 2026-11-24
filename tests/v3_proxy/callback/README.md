# Nested-callback LEAK→SAFE isolation regression

The compiled-artifact adversarial suite (`adversarial.rs`, `adversarial_more.rs`,
`adversarial_scope.rs`, `nested.rs`) existed at source commit `b1a8052d` under
this path. Every test was `#[ignore]`d with the reason "requires compiled
Program0 and callback-safe generic kernel", so it never ran in CI, and the
suite was later removed. Acceptance #2 still requires a non-vacuous nested
callback capability-isolation regression that CI actually runs.

At the reviewed head the regression is restored in a compile-free form:

- `program0/src/proxy/host/wire_test.rs::leak_safe_nested_callback_isolation`
  drives the generic callback dispatch with the `EVAL` opcode and with
  out-of-scope/private disk paths, and requires a denial in each case. It fails
  if the callback re-enters evaluation or if path hygiene is weakened.
- `.github/workflows/native-components.yml` runs it by name in the
  `native-components` job, so it is a real CI gate rather than an ignored test.

Scope: this is the skrynia-tools-side source regression. The kernel-side WAT
fixture `nested_guest_cannot_steal_root_disk_fd_after_callback` (root-FD
exfiltration via nested eval / multi-callback FD reuse) lives in the Skrynia
kernel repository and is **NOT RUN** by this crate: running it here would need
a kernel rebuild, which is outside this source-only task. Do not present this
unit test as proof of the compiled-kernel WAT regression.
