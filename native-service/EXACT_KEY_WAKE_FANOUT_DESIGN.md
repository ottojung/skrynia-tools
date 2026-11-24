# Generic exact-key wake invalidation (development integration)

The native P1 provides an invalidation channel for programs with many mutable
objects. The kernel and Program0 remain generic; neither knows applications,
object names, Web Push providers, subscriptions or credentials.

## Same-namespace writes

The reserved exact object key is `skrynia-wake-v1` (`push::WAKE_KEY`).
When a namespace has subscribers to this key, a successful AppPut or AppDelete
stages an invalidation under the mutation lock **before** committing its
actual object. A direct write of the reserved key uses the same fixed slot.
A sender may receive a harmless spurious invalidation after a failed object
commit; its consumer must reread authoritative state.

## Cross-namespace writes

The writer of the *wake* namespace issues a `Kind::Wake` grant restricted to
the reserved exact wake key and the read-cap identifier of one source writer
namespace. The source writer needs its own write cap and this narrowly scoped
wake-only grant, but **never** the wake namespace's write cap or sender grant.

`AppPutWake` (opcode 25) and `AppDeleteWake` (26) verify their source
CAS precondition and Wake grant under the same mutation lock. Both the
source and target wake generations are durably staged before the source
write/delete. No post-commit best-effort secondary PUT exists. Legacy
AppPut/Delete remain valid unchanged. `PushWakeGrant` (24) is the
issuer-only operation. Wake authorization cannot write ordinary target data.
Wrong key, foreign source namespace, revoked grant and ungranted issuer fail
closed. There are no prefix subscriptions.

## Bounded delivery

The reserved wake key uses a **fixed durable generation slot** keyed by exact
(namespace, key), not one hint file per mutation. Every subscribed mutation
advances a monotone generation and marks the slot dirty. The existing sender
list/read/ACK protocol returns the latest pending generation, an opaque
32-byte invalidation and a generation-conditional ACK. An ACK of g cannot
clear g+1, including after restart or when the sender rereads an old hint.

The slot remains constant-size even after more than 1,024 offline mutations.
A failed durable slot write aborts the source mutation; a healthy-disk
notification backlog does not. For **ordinary non-wake exact data keys**, the
legacy bounded, per-sequence hash hint queue is preserved when it can accept
writes; a separate generation-conditional dirty slot acts as an overflow or
crash fallback. A full/corrupt old hint queue can neither make an authorized
source CAS fail nor lose a later committed change after the sender ACKs
already-read older entries. Both kinds of hints are invalidations, not event
logs, and clients must re-read authoritative state.

## Acceptance and remaining release work

The native-service integration tests cover independent writer namespaces,
113 dynamic threads, over 1,024 offline changes, sender read/mutate/stale ACK,
grant replacement and false-authority cases, and spurious pre-commit hints.
Rust structural lint (50 code lines/file and 10 entries/directory), default
rustfmt and native-service Clippy are mandatory.

This is a source-level development contract. Compile and pin an updated
ordinary P1 WASM binary, integrate the client wake-grant provisioning and
AppPutWake/AppDeleteWake, then exercise the real sender and browser with
independently authorized credentials before claiming real push delivery.
No production/TLS/DNS/data migration follows automatically from a code merge.
