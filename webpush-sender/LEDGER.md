# Durable replay ledger (sender-side, DRAFT)

## Placement: sender, not P1

The ledger lives in the **standalone webpush-sender crate**. P1 is a generic
object/CAS server: adding WebPush delivery state to it would couple the kernel
front to one application and violate the "v3 kernel never gains WebPush/VAPID
code" boundary. P1 already owns the authoritative hint queue and an idempotent
`HintAck`; the sender owns *its own* local knowledge of "I delivered this hint
and persisted proof before I claimed the ACK". Keeping the ledger here means no
P0/P1/kernel/main changes and no shared mutable state.

No unsecured shared path: the production store is constructed by
`FileLedgerStore::from_config(dir, cfg)` from an **operator-supplied directory
capability** and the one public `ledger_binding(cfg)` derivation used by both
the filename and the in-file header, so they cannot diverge. Raw `sender_cap`,
`ns` and `key` never appear in a path or log. `dir` must be private (0700) and
single-writer; it is never shared with another sender instance. `Pump::new`
uses an in-memory store for hermetic tests only; production must inject
`FileLedgerStore` through `Pump::with_ledger` / `with_https_transport`.

## Binding and boundedness

`binding = SHA256("skrynia-webpush-ledger-v1" || sender_cap || ns || key ||
ledger_epoch_le)`. `ledger_epoch` is the explicit migration escape hatch: bump
it when P1 hint highwater is reset/migrated. A different grant or epoch fails
`open()` with `Config("ledger-binding")` instead of inheriting stale state.

The file is `magic || binding || cap || count || [seq:u64,state:u8,pad:7] ||
tag`, where `tag = SHA256("skrynia-ledger-mac-v1" || sender_cap || body)` is a
**keyed** integrity tag: the 32-byte sender bearer capability is the MAC key and
never appears in the path or the file, so directory write alone cannot forge or
copy a valid image from another scope. It is bounded to `MAX_HINTS` (1024)
records; at cap the oldest record is evicted, so recording never silently stops.
Magic, binding, keyed-tag, shape and duplicate-seq checks make corruption,
forgery and foreign files fail closed (never "treated as empty").

Commit is atomic: write temp, `fsync` file, `rename`, `fsync` parent directory.
`save` returns only after durability; ENOSPC maps to `Error::Quota`.

## Crash-point matrix

| Crash point | Ledger state | Replay behaviour | Delivery |
| --- | --- | --- | --- |
| successful remote send, before marker fsync | no record | hint reprocessed and re-sent | at-least-once (inevitable) |
| after marker fsync, before ACK RPC | `Delivered` | `Decision::Delivered` -> **re-ACK, no re-send** | exactly one send |
| ACK RPC in flight / lost reply | `Delivered` | idempotent re-ACK; no re-send | exactly one send |
| ACK accepted, marker->Acked fsync lost | `Delivered` | re-ACK (harmless), no re-send | exactly one send |
| `Delivered` -> `Acked` committed | `Acked` | `Decision::Acked` -> skip | no duplicate |
| restart, same grant | loaded and checksum-verified | state restored; no duplicate | safe |
| corrupt file / quota full | load/save error | `open` or `mark_*` fails closed; **no ACK without a durable marker** | safe, possible duplicate |
| seq wrap/reuse under same binding | seq <= max but unknown | ledger reports `Error::Wire("ledger-reuse")`; the pump isolates it **per hint** and treats the still-listed hint as `Fresh`, re-delivering it (at-least-once) instead of aborting the queue | no false suppression, no queue stall |
| P1 highwater reset/migration | epoch bumped -> new binding | old file refused; new file starts empty, hints delivered anew | no false suppression |
| unrelated sender grant | different cap/ns/key/epoch | binding mismatch, fail closed | no cross-grant poison |

At-least-once delivery across the pre-marker crash is unavoidable without a
two-phase commit spanning the remote endpoint; **no exactly-once claim** is made.
`src/pump/retry.rs:33` does consume `Reply.retry_after_ms` with bounded backoff
capped at 60 s; the earlier "deliberately untouched" claim is false at this
head and has been withdrawn. An ACK still requires a durable pre-marker.

## RED / GREEN evidence

RED at base `69096325`: the durable API (`Pump::with_ledger`, `MemoryLedgerStore`,
`FileLedgerStore`, `Decision`) does not exist, so the fault-injection cases do not
compile; the old `pump/ledger.rs` `HashSet` cap silently stopped recording at
1024 and `cycle.rs` marked only after `HintAck`, with no persisted pre-ACK
marker and no replay-ACK path.

GREEN at this branch: 10 unit invariants (bounded growth, restart, corruption,
foreign grant, seq reuse, epoch migration, quota/load faults, real atomic
fsync+rename commit) plus 2 end-to-end fault cases (quota-withholds-ACK,
lost-ACK replay without re-send), plus the re-review fixes: a multi-hint
out-of-order regression (seq 5 fails, seq 6 succeeds, then seq 5 recovers with
zero starvation — `tests/retry.rs`), a real `FileLedgerStore` write/restart
round trip with an unkeyed-checksum forgery rejection (`src/ledger/wire.rs`),
and public `ledger_binding(cfg)` / `FileLedgerStore::from_config` constructors.
Each `#[test]` is independently executable by name.
