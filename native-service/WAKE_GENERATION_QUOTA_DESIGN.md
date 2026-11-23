# Bounded durable wake generation vs. data-plane liveness (DRAFT, DEV-only)

Stacked on PR89 (`04ef1ab6`, exact-key wake fan-out). Source-only; no WASM
rebuild, no deployment, no main/prod/TLS/DNS/Ivanka changes, no real data.
Independent of the cross-namespace wake agent (`AKwTA9I1DtMCBrsBA1WpTs-fort`,
worktree `/tmp/opencode/countries-crossns-wake-5z`); no files or branches of
that agent were touched and nothing was merged.

## 1. Source-grounded defect

PR89 stages a reserved `skrynia-wake-v1` hint for **every** appdata mutation
inside the mutation lock (`ops/write/put.rs`, `ops/write/del.rs`) via
`push::enqueue_hints`. `push/hints/queue.rs` counts the pending hint files at
that exact key and returns `PushErr::Quota` once `existing.len() >= MAX_HINTS
(1024)`. Both write paths map `Quota` to `AppErr::Busy` *before* committing the
data write.

Consequence: an offline sender/provider lets 1024 mutations accumulate, then
every unrelated namespace write fails with `Busy` solely because a wake
subscriber exists. With cross-namespace wake fan-out the whole shared app
becomes unwritable. This is a data-plane write outage caused by a notification
backlog.

Executed RED evidence (unmodified PR89, isolated target dir, offline):
`cargo test --offline -p skrynia-native-service --test wake_quota_liveness`
gave `wake_quota_never_blocks_ordinary_data_writes` left=1024 right=1088
(exactly the 1024 quota cut), plus the two coalescing assertions failing with
2 hints where 1 was required. That test file is committed as the regression
gate; its assertions were recorded before the source fix existed.

## 2. Design: generation == durable hint seq, coalesced per exact key

`push::mark_wake_dirty` (new, `hints/queue.rs`) replaces the per-mutation wake
enqueue under the *same* mutation lock:

1. If the exact wake key has no subscriber, do nothing (unchanged early-out).
2. `seq::reserve` advances the persistent high-water counter for that exact
   `(namespace, wake key)`; the returned monotone value **is** the dirty
   generation. The counter survives ACK and restart.
3. Write exactly one pending hint at `hint_path(..., generation)` durably
   **before** removing any older wake hint files.
4. Remove all other pending wake hint files.

Result: wake storage is O(1) per exact key (one hint file plus the counter),
independent of mutation count and provider uptime. There is no quota check on
the wake path, so a full/backlogged wake state can never turn into
`AppErr::Busy` for an ordinary data write.

Why generation-as-seq gives conditional ACK without a wire change:

* The sender already receives the pending seq from `hint_list`, reads that hint,
  and calls `hint_ack(seq)` (P1 `OP_HINT_ACK`, unchanged wire schema). The seq it
  ACKs is the generation g it observed.
* A mutation after the sender's read advances the counter to g' > g and its hint
  lives at a distinct path `hint_path(..., g')`; the stale path g is removed by
  coalescing. `hint_ack(g)` can therefore only delete the file at g. It can
  never clear a newer pending generation at g'. The "ACK(g) clears only while
  current == g" rule is enforced structurally by path identity and monotonicity.
* There is deliberately **no** "pending exists => skip" shortcut. Every
  mutation advances the counter; a sender that read g and ACKs after a mutation
  leaves g' pending. No wake is lost in the read/mutate/ACK race.
* `hint_ack` remains idempotent: a replayed seq whose file is already gone
  returns `Ok`. A crash-leaked stale file at g is harmless; delivering it is a
  spurious wake (senders re-read authoritative state), and the ACK drops it
  without touching g'.

## 3. Correctness argument

* **Reliable eventual invalidation:** after the last mutation at generation g,
  a durable hint at g exists and is listed by `hint_list`. A sender that
  re-reads authoritative state computes the full current view; coalescing many
  mutations into one wake is sound because the wake is an invalidation, not a
  per-event log.
* **No write outage:** the wake path performs a constant number of disk ops and
  never consults `MAX_HINTS`. 1088 mutations with an offline sender all commit.
* **Race t1/t2/t3:** sender reads g at t1, mutation g' at t2, ACK(g) at t3 —
  hint at g' remains and is delivered next cycle. Tested explicitly.
* **Restart / ABA:** the counter is durable; generation is strictly monotone
  until `u64` exhaustion (a documented hard residual). A fresh sender grant /
  sender process over the same disk sees the same generation; an ACK from an old
  sender cannot clear a newer one. `scope()` still binds one exact `(ns,key)`
  sender grant (no prefix), and the webpush ledger binds `(cap, ns, key, epoch)`.
* **Crash ordering:** counter advance -> new hint write -> old hint prune. A
  crash between steps leaves extra stale hints (spurious wakes) but never a
  missing hint for the latest generation, because the new hint is durable
  before any older one is removed. A crash before the data write leaves the
  normal spurious wake already accepted in PR89.
* **Bounded storage / exact-key caps:** one wake hint per key; `hint_list` still
  truncates to `MAX_HINTS`, `sub_fetch` to `MAX_SUBS`, and the sender ledger to
  1024 records. Those caps are untouched.

## 4. Tests (run offline, isolated target dir)

`native-service/tests/wake_quota_liveness.rs` and
`native-service/tests/wake_gen_restart.rs` (new, 4 tests total; split to satisfy
the 50-code-LOC per file structural lint):

1. `wake_quota_never_blocks_ordinary_data_writes` — MAX_HINTS+64 real puts over
   actual `ops::put`/`Mem` all commit; data durable; one pending hint.
2. `stale_ack_after_mutation_retains_new_wake` — read g1, mutate to g2, ACK(g1),
   assert g2 still pending, then ACK(g2) clears.
3. `generation_is_durable_for_a_fresh_sender_after_restart` — a second sender
   grant over the same disk sees/acks the monotone generation; its stale ACK
   cannot clear the newer wake.
4. `replayed_ack_is_idempotent_and_never_removes_current` — a crash-leaked
   stale hint is cleaned, and replayed ACKs are no-ops.

PR89's own tests were updated to coalesced expectations (one hint, generation
equals mutation count): `wake_fanout.rs`, `wake_fanout_authz.rs`,
`wake_fanout_neg.rs`, and `wake_fanout_quota.rs` (the latter's "quota rejects
mutations" assertion is intentionally superseded — that behaviour was the bug).

GREEN after fix: `cargo test --offline -p skrynia-native-service --all-targets`
— 27 unit + 35 integration + 12 wake tests, 0 failed. Full workspace gates and
source-lint checks are recorded in the board comment.

## 5. Sender durable ledger / grant check (verified, no change needed)

* `hint_ack` idempotent for replayed seq (`sender/read.rs`); pump re-ACKs a
  DELIVERED marker without re-sending (`pump/cycle.rs`), `mark_acked` is an
  idempotent state write (`ledger/update.rs`).
* Ledger records are bound to exact `(cap, ns, key, epoch)` with a keyed MAC;
  foreign/rotated grants fail closed (`ledger/model.rs`, `ledger/tests2.rs`).
* `DurableLedger::decide` suppresses only exact seq matches; unseen seqs stay
  Fresh. Monotone generations mean old evicted records cannot suppress a new
  wake. No sender-side source change is required for this fix.

## 6. Hard residuals / NOT claimed

1. No P1 WASM/Podman/browser rebuild or real provider acceptance; release stays
   NO-GO. This is a source-level DRAFT only.
2. Direct subscriptions on ordinary *data* keys still use the per-mutation
   `enqueue_hints` and its MAX_HINTS fail-closed quota. Only the reserved wake
   key (the PR89 fan-out path) is coalesced. Cross-namespace exact-key grants
   remain the crossns agent's scope; this PR does not implement or merge them.
3. `u64` generation exhaustion returns `Quota` and would fail writes again; at
   one increment per mutation this is not reachable in practice but is an
   unbounded-run residual.
4. A sender that ACKs a seq it did not actually deliver could clear the current
   generation; this assumes the local sender reports the seq it really read,
   as the current pump does. No adversarial sender defence added.
5. Crash between counter bump and new-hint write can skip an unobserved
   generation number; that is safe coalescing because no sender had seen it.
