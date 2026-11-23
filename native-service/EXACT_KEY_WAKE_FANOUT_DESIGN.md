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

## 6. Cross-namespace Wake grants (PR89 follow-up, DEV-only)

PR89's `WAKE_KEY` fan-out is same-namespace only: `read_cap = SHA256(write_cap)`
derives disjoint namespaces. Along fullstack generates independent random writer
caps for catalogue, tes, vitalik, so one WAKE subscription cannot observe
mutations in another namespace. PR89 is source-correct for its local invariant
but not a complete release-blocker fix.

This follow-up adds an explicit, least-privilege Wake-only grant: the wake
namespace writer issues `Kind::Wake` bound to `(wake exact key, source writer
namespace ID)`. The source writer calls `AppPutWake`/`AppDeleteWake` with
`(source write cap, key, expect, value, wake grant, wake key)`. Under ONE P1
lock the service validates source authz+CAS, verifies the Wake grant for that
exact source namespace and exact wake key, enqueues an opaque durable hint at
the destination wake key BEFORE source commit, fails closed on invalid
grant/quota, then writes/deletes source. Existing AppPut/Delete unchanged. No
prefix subscriptions, no P0/kernel/Program0 semantics, no app-specific logic.

Security negatives covered by source tests: disjoint writer namespaces (positive
cross-ns wake), foreign source namespace, wrong wake key, fake/unauthorized
grant, revoked/replaced grant, concurrent revoke under lock, failed commit
leaves only spurious wake, MAX_HINTS quota fails closed.

## 7. Availability coupling / generation-coalescing gate (owner 19:31Z)

IMPORTANT, owner-decision point - not decided unilaterally here.

With a global wake subscription present, every mutation stages a durable wake
hint. `hints/queue.rs` returns Quota at MAX_HINTS=1024 and put/delete map that
to Busy BEFORE committing source value. If an external provider/sender is down
for 1024 mutations, unrelated data writes become unavailable due to
notifications. A shared cross-namespace wake key magnifies this into
system-wide data-plane availability coupling. This is not a low caveat.

Options (owner acceptance required):
- A) Keep fail-closed quota (current). Preserves "no committed mutation without
  queued wake" but couples data-plane availability to sender liveness.
- B) Durable dirty-generation/coalesced invalidation per exact wake key. Under
  the same mutation lock, increment generation / write pending dirty state
  BEFORE appdata commit. Sender reads/dispatches generation g; ACK removes only
  if current generation is still g (otherwise another wake remains pending).
  Mutations during in-flight delivery cannot lose a notification. Requires
  sender-ledger/wire changes, restart/idempotence, revocation and
  crashed-before-commit spurious-wake coverage. MUST NOT be naive "skip enqueue
  when one hint exists" because a mutation after sender delivery but before ACK
  would be lost forever.

This change set keeps PR89 fail-closed behavior unchanged and does not
implement option B; it flags the decision. Exact-key-only grant scoping, no
prefix, no P0/kernel special logic preserved.
