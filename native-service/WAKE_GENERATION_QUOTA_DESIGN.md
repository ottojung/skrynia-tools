# Bounded wake generation with conditional acknowledgement

**Current implementation:** `push/hints/wake/{format,ops,sender}.rs`,
`push/hints/queue.rs`, `push/sender/{list,read}.rs` and
`appdata/ops/write/{put,del,wake_hint}.rs`.

For the reserved exact key `skrynia-wake-v1`, the server stores one 16-byte
atomic slot: `WSL1 || u64_le(generation) || dirty_byte || 3 zero bytes`.
Unknown or corrupt formats fail closed. Under the mutation lock, the server
writes the next generation with dirty=true **before** source commit.
A sender lists only the current dirty generation g, reads a generic 32-byte
invalidation, and ACK(g) clears dirty only if the stored generation remains g.
Mutation g+1 between sender read and ACK therefore remains pending.

There are no per-generation wake hint files and no prune/remove step on the
write path; failure to remove an old file cannot disable ordinary writes
or hide the latest generation behind a 1,024-record listing limit.
With no subscriber the wake path does nothing; source CAS and legacy exact-key
data hints are otherwise unchanged. A failed durable slot update prevents
source commit. A crash between slot update and data commit can leave a spurious
wake only. Generation overflow is an explicit, practically unreachable
fail-closed condition.

## Ordinary data-key compatibility and loss-free overflow

The legacy exact-object per-sequence hint queue is still bounded by
`MAX_HINTS=1024`; success initially produces the same hash/delete wire
payload and monotone ordinary seq number as before. For a given key, the
service attempts new legacy per-file hints only while the monotone sequence
counter is below **256**. Past that threshold it uses the durable coalesced
fallback, even if old records have since been ACKed. This explicit finite
budget bounds per-request directory scanning and WASM fuel usage. The
sender must interpret any fallback as an invalidation, not as an event hash.
**Before every CAS**
having an exact-key subscriber, P1 additionally advances a durable,
constant-size generation slot under the source mutation lock. It also stages
the source/shared reserved wake slot(s) as applicable. If the CAS succeeds
and a legacy per-key hint can be durably enqueued, P1 conditionally ACKs the
temporary slot (still under the lock), preserving the original sender view.

If the per-key queue is full, corrupt, inaccessible or the process dies
after source commit but before enqueue/slot-clear, the dirty slot remains.
A scoped sender sees **one** current dirty generation with the high sequence
bit (`1<<63`) set, reads 32 opaque zero invalidation bytes, and ACKs only
that exact generation. The sender does not have to list corrupt/full
legacy hint files to discover it. The high-bit namespace is distinct from
legacy numeric seq files; old hints resume being listed after the fallback
is ACKed. They can be spurious duplicates and are drained normally.

This closes the exact-key race: a sender may read/deliver 1024 old queue
entries, then an authorized source commit occurs while those entries await
ACK, then sender ACKs all old entries. The newer fallback remains pending,
and a stale fallback ACK cannot clear a generation created by yet another
mutation. A full legacy notification queue is **never** a reason to reject
a data CAS. Slot-write or corrupted-slot failure remains fail-closed,
because otherwise no reliable invalidation can be promised.

The normal 16-byte slot representation does not change; ordinary data-key
fallback IDs reserve bit 63, so their internal generation domain stops at
`2^63-1` (practically unreachable). Existing sender wire methods and
scoping grants are unchanged. The sender *must* accept full `u64` seqs and
treat the 32-byte fallback as an invalidation, never a definitive deletion.

The native-service wake and quota suites cover direct reserved-key put/delete,
restart/corruption, conditional stale ACK, cross-namespace quotas and over
1,024 mutations. They additionally prove the **non-vacuous** read-old-1024 →
new source commit → ACK-old-1024 lost-notification RED on the old source, and
GREEN with the new dirty fallback; repeated fallback generations and legacy
queue restoration are also tested.

**Not evidence of production delivery:** the ordinary P1 WASM and browser
client must be rebuilt and verified against the integrated ABI; real provider
delivery, operational credentials, five-app migration, TLS and release
qualification remain separate acceptance gates.
