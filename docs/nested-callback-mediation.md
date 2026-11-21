# Nested callback mediation for trusted ordinary service P1 (proposal)

Issue: Antonina `8TasxocIctWh2VHtJ7e4EX-temperate` (follow-up to PR47/PR48, service `ed2MwvKbbtdHjMZi7ITTx5-dolokhov`, canonical kit `w4wgIqJnlASKGYrl1UGd8H-realise`).

Status: **design/proposal, not a merge-ready implementation**. No P0 application/prefix policy added. No duplicate kernel builds. Pinned kit coordination required.

## Goal

Enable an ordinary trusted service **P1** (cb-guest native-service) to evaluate an arbitrary untrusted **P2** and mediate P2 `skrynia.invoke` callbacks through P1's own service logic (e.g. `appdata::cb::guest_callback`, namespace-scoped to the invoked app), without granting P2 raw/root filesystem or ambient authority and without adding policy to **P0**.

## Studied sources

- Kernel `62ea225858f2a5d6a08cde6c9b1b9abbbbb0cb2e` (`feat/generic-mutation-lock-syscall`, includes #111/#112 context): `rust-server/src/boot/vm/host/eval/*`, `boot/vm/link/{root,child}.rs`, `boot/vm/state/*`, `host/eval/guest/{exec,link,inv,call,fds}.rs`.
- Kernel #111 `513bb8f` / `99dbe5f` (`fix/generic-eval-callback-scope`): callback runs with parent authority by clearing guest cap (`guest.take()`) and restoring saved cap after callback/trap; staging in root memory.
- Kernel #112 `62ea225`: root-only `try_mutation_lock`/`release_mutation_lock` (1 acquired, 0 contended), request-owned, trap-safe release.
- Tools #40 (`feature/v3-generic-nested-eval`): P0 callback-relay EVAL opcode 13, layout `OPEN_DISK..RANDOM_FILL,EVAL` (1-13), path-based eval transferring FDs with same P0 callback.
- Tools #42 (`feature/v3-generic-mutation-lock-relay`): adds relay opcodes 14/15 `MUTATION_TRY`/`MUTATION_RELEASE` on #40 layout.
- Tools PR47 base `fd08a3a` / PR48 `282ed2b`: cb-guest wire/layout `...SYNC=10,RANDOM=11,TRY_LOCK=12,RELEASE_LOCK=13` (no EVAL), service-local mediation via `registry/reg_ops/invoke.rs` + `appdata::cb::guest_callback`.

## Current blocker (precise)

1. **P1 capability boundary:** P1 cb-guest imports only `skrynia.invoke` to P0 (`native-service/src/cb_guest/cap/call.rs`). It has no direct kernel `eval`.
2. **Root-only eval:** kernel links `eval` only in root (`rust-server/src/boot/vm/link/root.rs` -> `host::eval::add`). Guest execution (`host/eval/guest/link.rs`) provides only stream `read/write/close` and `skrynia.invoke`.
3. **Callback target fixed to eval caller:** `host/eval/cb.rs` resolves callback from caller's `__indirect_function_table`; `guest/link.rs` wires child `invoke` to that callback func.
4. **Callback staging assumes root:** `host/eval/guest/call.rs` stages request/response in `frames[0]` memory (`root_inst`) and copies child response back; this is correct only when root is provider.
5. **Callback authority restore assumes root:** callback clears `vm.guest` (`None`) for execution and restores saved child cap (#111). `None` means unrestricted fd-table access (`host/fd/guard.rs`), correct for root provider but a privilege escalation if a guest provider's callback were run that way.
6. **P0 relay:** `program0/src/proxy/entry.rs` evals root guest with `host::callback_index()`; P0 host cb is a fixed transparent syscall relay. If P1 asks P0 to eval P2, P0 cannot supply P1's callback without a generic trampoline/re-entry protocol.

Result: P1 cannot receive P2 callback upcalls for mediation (`appdata::cb::guest_callback`) under current ABI without P0 policy or unsafe raw authority for P2.

## Proposed minimal generic extension (for kernel ABI coordination)

### A. Mediated guest-eval capability (preferred direction)

A generic, opt-in `eval` capability for trusted ordinary guests that need to orchestrate/mediate children:

- **Link/capability:** provide `skrynia.eval` to designated mediator guests (generic link mode), keep untrusted P2 on invoke-only + stream fds.
- **Provider instance:** record eval caller as callback provider (instance/memory), not `frames[0]`.
- **Staging:** callback request/response buffers in provider memory; child request/response in child memory (bounds/growth checked).
- **Authority during callback:** restore **provider's** execution capability (its guest cap/context), not `None`; never grant root/global fd access. Nested callback stack must be explicit.
- **Safety:** preserve #111 FD isolation across nesting, trap/fuel/cancel cleanup, depth/FD/compile limits, mutation-lock ownership rules, and no cross-context fd leaks.

### B. P0 trampoline / parent-callback channel (alternative)

P0 remains eval executor but kernel/P0 provide a generic suspend-reenter or message-channel mechanism to deliver P2 callback events to P1 and resume P2 with P1's response. Simpler capability story but higher re-entrancy/protocol complexity.

## Opcode/ABI reconciliation note

PR47-base relay numbering differs from #40/#42 stack (locks at 12/13 vs 14/15, no EVAL). Any new relay opcode/capability must be defined against the integrated kit ABI branch, not ad-hoc in this base.

## Narrow proof artifacts in this environment

- Tools commits on `agent/temperate-callback-mediation`: proposal + analysis (this doc), preserving PR48 fixes (`282ed2b`, `7ca12bb`) and host tests.
- No kernel source modification committed to shared kernel worktrees/branches; no kernel builds duplicated.
- Local kernel PoC patch may be prepared in an isolated throwaway copy for review, but end-to-end validation requires pinned kit and kernel ABI coordination.

## Validation status

- `cargo test -p skrynia-native-service --locked`: **2 unit + 35 integration PASS** (host/native, unchanged from PR48 baseline after this proposal commit set).
- `cargo fmt --all -- --check`: PASS.
- Not run: source_lints self-test, strict clippy all-targets, rustdoc, wasm32 cb-guest build (target std unavailable).
- Not possible here: real compiled wasm32 P0/P1/P2 mediated-callback execution against pinned kernel 62ea225/ABI-coordinated kit, malicious out-of-scope I/O denial runtime proof, trap/restart/concurrency kit evidence.

## Next coordinated steps

1. Agree kernel ABI (Option A/B) on canonical kernel branch with agents owning #111/#112/HTTP pump/compiler safety (no rebase interference).
2. Implement kernel callback-provider staging + capability-safe restore + mediator eval capability; add WAT security tests (P2 cannot access P0/P1/root fds, P1 mediates namespace callbacks, trap-safe).
3. Reconcile tools P0 relay opcodes with integrated stack; implement cb-guest mediated Eval using P1 callback endpoint; adapt Invoke path to use mediated callback.
4. Run pinned-kit end-to-end tests: successful registered invoke, denied out-of-scope I/O, concurrency CAS/lock behavior, trap/restart/lost-ack fail-closed.
