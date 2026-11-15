# Client protocol catalog and proof obligations

**Status: proposals, not implemented.** These protocols build on Skrynia's actual single-key primitives. None can increase server-enforced authority.

| Module | Function | Primitives | Priority |
| --- | --- | --- | --- |
| Raw client | bytes, grants, listing, CAS and push | HTTP operations | P0 |
| `CasCell<T>` | versioned one-key state | byte-SHA256 CAS | P0 |
| `AppendLog<T>` | ordered immutable stream | create-only slots | P0 |
| `EventSet<T>` | concurrent unordered contributions | create + list | P1 |
| `SnapshotStore<T>` | multi-object logical snapshot and transaction | immutable records + CAS head | P1 |
| `SealedEpochLog<T>` | retained log with registered consumer cursors | grants + CAS + delete | P2 |
| `TaskQueue<T>` | collaborative tasks and recovery | CAS + log | research |
| Signed/encrypted records | independently validated trust/confidentiality | client cryptography | separate design |

Each protocol requires: versioned wire schema, exact authority bundle, creation/bootstrap steps, correctness and liveness assumptions, bounded-work behavior, unknown-outcome recovery, golden interoperability fixtures, crash injection and adversarial tests.

## CAS cell

Read authoritative bytes, compute their SHA-256, then use conditional replace. Embed a strictly increasing revision and unique transaction ID when an application needs to avoid ABA even if payload values repeat. After 412, re-read and **re-evaluate** the update rather than blindly retrying the previous bytes. After an uncertain response, inspect transaction ID/history or return `OutcomeUnknown`. A user callback may have external side effects; do not automatically rerun it without an explicit contract.

## Append-only log (create-only)

Initialize a genesis key in a unique epoch; cooperating writers discover the first missing **consecutive** successor with fresh reads and race `POST` at that exact key. 201 is the append's commit, 409 advances to the successor, and a lost response requires checking the attempted key's **operation ID** before deciding to retry. Readers replay the contiguous prefix and stop at the first currently missing key; that is not a permanently sealed end.

Writers must be limited to **create only**, not modify/delete, and the live epoch must not be collected. This ordering proof requires cooperating writers, no deletion and fresh observations. A malicious writer can occupy future slots, insert invalid bytes or exhaust quota; the Rust API cannot enforce its algorithm on others. Tail hints and listing can accelerate scans but do not certify missing predecessor slots. Expose explicit positions (epoch, sequence), bounds and cancellation. A linear scan from genesis must not be disguised as constant-time.

## Logical transactions / snapshots

Create new immutable data and a manifest before publishing a **single** small CAS root. Readers pin one root, then traverse only that manifest. A successful root CAS is the logical commit point; 412 triggers full transaction reevaluation and uncertain commits need transaction-ID evidence. Garbage collection of old manifest/data is unsafe without protecting staged writers and in-progress readers, so retain versions until an explicit proven reclamation strategy exists.

## Garbage-collected append log

Never naively delete a currently acknowledged prefix: a stale writer can reuse the freed slot. Instead, establish disjoint epoch grants; revoke **all** create authority to an old epoch, observe its last committed contiguous slot after revocation, publish a sealed descriptor via CAS control, and only then activate the next epoch. The coordinator owns retention state. Registered consumers durably acknowledge positions; commit the retention floor under the same control as enrollment, and delete **only sealed epoch** entries below the floor. A reader must not infer its beginning from the smallest physical key during partial deletion.

Offline readers can block collection; dead coordinators can block rotation. Overlapping broad grants can defeat the seal. Unknown revocation outcomes and crashes between each step require recovery rules. Implement only after property/model tests validate revocation ordering, stalled readers, phantom slots, replays and malicious writers.

## Contributions, queues and notifications

An unordered event set can use unique create-only keys and scoped listing. Lexical key order is **not** chronological order. A task queue needs independently specified lease, claim, retry, ack and external side-effect rules; Skrynia itself guarantees no scheduler execution. Web Push messages are coalesced duplicate-prone wake-ups; read durable state to discover real changes.

## Validation levels

Distinguish: (1) enforced constraints on all authorized callers, (2) invariants assuming cooperating participants, and (3) accepted data proven by reader verification. For every high-level API publish which level each guarantee inhabits. Signatures can detect corrupted/unauthorized payloads according to a trust policy, but cannot stop an authorized holder from deleting data or refusing progress.
