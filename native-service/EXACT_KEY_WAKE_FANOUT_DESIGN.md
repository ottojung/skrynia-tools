# Exact-key wake fan-out for ordinary P1 (DRAFT, DEV-only)

Status: source-level design and tests only. Not a release artifact. No WASM
binary was rebuilt, no deployment, no main/prod merge, no real keys or data.

## 1. Source-grounded RED (actually executed)

`native-service/src/registry/appdata/ops/write/put.rs` enqueues a change hint
only at the exact key being mutated; `push/hints/queue.rs` lists subscribers
at that exact key and returns early when there are none. Native push grants
are exact-key (`push/grant/mod.rs`). Therefore a single subscription cannot be
woken by mutations of any other key.

`native-service/tests/wake_fanout.rs` proves this with two distinct keys:
subscribe only `traveller-tes-v1`, mutate `thread-0042-tes-v1` through the real
`appdata::ops::put`, and assert zero hints on the subscribed key (and zero
anywhere) while the value change is durable. On the unmodified base the
companion test `wake_object_is_invalidated_for_every_mutation` FAILS (RED).

Recorded run (isolated target dir, no heavy build):
`cargo test --offline -p skrynia-native-service --test wake_fanout` —
1 passed / 1 failed, exit 101.

## 2. Design evaluation

* (a) N exact-key subscriptions for all traveller and dynamically allocated
  `thread-{profileId}-{author}` keys. Rejected: the key set grows to hundreds
  (113 profiles in the catalogue) and is open-ended ("future keys"). Every new
  key needs a fresh app-scoped grant + subscription + sender-scheduler entry;
  revocation/identity rotation is O(N); disk and sender count grow linearly.
  Cannot bound resources or guarantee coverage of a key not yet known.

* (b) Atomically coupled invalidation of a designated exact wake object under
  the SAME mutation lock/commit. **Chosen.** A namespace-scoped reserved exact
  key `skrynia-wake-v1` is the single wake object. Any successful `put` or
  `delete` in that namespace enqueues an invalidation hint on that exact key
  inside the existing `with_lock` closure, immediately before the object
  write/delete. The app holds ONE ordinary exact-key Subscribe grant and ONE
  subscription/sender for the wake key; the sender re-reads authoritative
  state (already implemented in the Along SW chain) and suppresses false
  notifications. No prefix subscriptions, no best-effort second PUT, no
  polling, no app semantics in kernel/P0, no new opcode, no new authority.

* (c) Other designs considered: (c1) consolidate all app state into one object
  — would force a large-object/chunking rewrite and breaks the existing data
  model; (c2) userland wake-first double PUT — NOT crash-equivalent: the sender
  may consume the wake hint before the data PUT commits and then suppress a
  genuine change permanently (a lost wake); (c3) prefix/wildcard subscriptions
  or unconditional `showNotification` — explicitly forbidden by owner policy.

## 3. Why (b) is correct and minimal

* Crash safety: both hints (mutated key and wake key) and the object write are
  in one `with_lock` scope over one lock. If the write commits, the wake hint
  is durable. If a crash precedes the write, the wake hint is spurious only;
  senders MUST re-read authoritative state (documented in `hints/queue.rs`),
  so no false notification is claimed. There is no second PUT after commit.
* Least privilege: fan-out targets only the wake key of the writer's own
  namespace; reachability still requires a valid exact-key Subscribe grant for
  that wake key. A foreign writer cannot create hints in another namespace.
* Bounded: one extra durable hint per mutation, under the existing `MAX_HINTS`
  quota; queue exhaustion fails the mutation closed rather than dropping a
  wake. One subscription/sender regardless of data-key count.
* General purpose: the rule is data-model agnostic and applies to every key,
  including keys allocated in the future. No traveller/thread/channel names
  appear in P1 or P0.

## 4. Negative/edge tests added (all executed, see board handoff)

* both traveller keys and a dynamic thread key wake the single wake object;
* 113 dynamically allocated thread keys with exactly one subscription;
* durability across a reopen; revocation stops fan-out;
* foreign-writer namespace isolation; delete also invalidates;
* crash before value commit => spurious wake only, no data mutation without a
  wake; bounded queue rejects mutations once `MAX_HINTS` is reached.

## 5. Remaining gates (NOT claimed here)

1. Rebuild the push P1 WASM from this source is an operator/ capacity-gated
   step; this work did not rebuild the pinned artifact `cf1322c3...`.
2. Along userland must issue the exact-key Subscribe grant for
   `skrynia-wake-v1` and subscribe to it (one subscription/sender), and keep
   the SW authoritative-state re-read/baseline logic already under review.
3. Real outbound VAPID provider + disposable-Chromium PushManager acceptance,
   operator-provisioned caps/bearer, restart/migration, five-app backed-up
   migration and independent exact-HEAD adversarial review remain open.
4. Source tests are NOT browser+backend acceptance; release stays NO-GO.
