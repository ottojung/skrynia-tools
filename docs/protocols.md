# Client protocol catalog and proof obligations

**Implementation status:** the native raw HTTP client, byte-SHA256 CAS cell, create-only append log, unordered event set and initial immutable-manifest snapshots exist. Other protocols below are design proposals. No client-side protocol can increase Skrynia-enforced authority.

| Module | Function | Primitives | Priority |
| --- | --- | --- | --- |
| Raw client (implemented) | bytes, grants, listing and CAS; Push not yet wrapped | HTTP operations | available |
| `CasCell` (implemented) | reads byte snapshots; conditional replace, re-evaluated update callback | byte-SHA256 CAS | available |
| `AppendLog` (implemented) | bounded contiguous reads and create-only appends with operation IDs | create-only slots | available |
| `EventSet` (implemented) | unordered, create-only byte contributions; bounded, non-snapshot scan | create + GET + list | available |
| `SnapshotStore` (initial) | byte-valued multi-object snapshot, immutable manifest + single CAS head | create + modify + GET | initial |
| `SealedEpochLog<T>` | retained log with registered consumer cursors | grants + CAS + delete | P2 |
| `TaskQueue<T>` | collaborative tasks and recovery | CAS + log | research |
| Signed/encrypted records | independently validated trust/confidentiality | client cryptography | separate design |

Before qualifying any protocol for production-grade guarantees, require: versioned wire schema, exact authority bundle, creation/bootstrap steps, correctness and liveness assumptions, bounded-work behavior, unknown-outcome recovery, golden interoperability fixtures, crash injection and adversarial tests. The initial implementations have unit tests and live-server smoke verification, but not all these stronger qualification gates.

## CAS cell

The implemented `CasCell` API reads authoritative bytes, computes their SHA-256 and uses conditional replace. The `update` callback is documented to be pure because 412 may rerun it. The current type is byte-oriented, not `CasCell<T>` with automatic typed serialization.  Embed a strictly increasing revision and unique transaction ID when an application needs to avoid ABA even if payload values repeat. After 412, re-read and **re-evaluate** the update rather than blindly retrying the previous bytes. After an uncertain response, inspect transaction ID/history or return `OutcomeUnknown`. A user callback may have external side effects; do not automatically rerun it without an explicit contract.

## Append-only log (create-only)

The implemented `AppendLog` uses fixed epoch `e000001` and twelve-digit slot keys. There is no current epoch rotation or garbage collection. Initialize a genesis key in that epoch; cooperating writers discover the first missing **consecutive** successor with fresh reads and race `POST` at that exact key. 201 is the append's commit, 409 advances to the successor, and a lost response requires checking the attempted key's **operation ID** before deciding to retry. Readers replay the contiguous prefix and stop at the first currently missing key; that is not a permanently sealed end.

Writers must be limited to **create only**, not modify/delete, and the live epoch must not be collected. This ordering proof requires cooperating writers, no deletion and fresh observations. A malicious writer can occupy future slots, insert invalid bytes or exhaust quota; the Rust API cannot enforce its algorithm on others. Tail hints and listing can accelerate scans but do not certify missing predecessor slots. Expose explicit positions (epoch, sequence), bounds and cancellation. A linear scan from genesis must not be disguised as constant-time.

## Unordered event set (optional)

The native `event_set::EventSet` stores arbitrary event **bytes** at `PREFIX.ID`, with 32 lowercase hexadecimal characters in each explicitly supplied ID (use independent random IDs or a domain-defined unique scheme). `insert(id, bytes)` uses create-only `POST`: a repeated ID with identical bytes succeeds idempotently, but a repeated ID with different bytes fails as a collision. When a create response is lost, the client reads the attempted key: identical persisted bytes establish successful publication; a missing key leaves an **uncertain** outcome, never an automatic second POST.

`scan(max_pages)` performs lexicographic, paginated listing and then reads each advertised key. It checks page ordering, key scope, ID grammar, and advancing cursors. It stops after at most 100 pages of 1000 keys and indicates `complete=false` if additional pages remain. A completed scan means only that the observed pagination had no next page: **it is not a consistent snapshot**. Newly created keys preceding a cursor can be missed. This data structure is grow-only only when all writers have *create-only* authority and no delete/modify capabilities overlap the scope; Skrynia itself does not enforce the event-set protocol beyond individual operations. Listing requires a user token with `list`; object reads are public. Unknown IDs are not automatically trusted or cryptographically authenticated.

`EventSet` is an optional client protocol for unordered contributions—not a mandatory app architecture, queue, log, globally ordered timeline, exactly-once stream, or garbage-collected store. It does not impose a JSON schema, event-processing strategy, or lifecycle on applications.

The [native example](../examples/event_set.rs) can run with `SKRYNIA_URL`, `SKRYNIA_EVENT_TOKEN` (a narrowly scoped `create` + `list` user token), and a fresh `SKRYNIA_EVENT_PREFIX`. The optional [live integration test](../tests/ssh/protocols_live.sh) starts an isolated real Skrynia server, issues a child grant and validates create, idempotency, collision rejection and listing with no authority to modify or delete. Run it by setting `SKRYNIA_TEST_SERVER_BIN` to the full path of a previously built compatible `skrynia-server` executable. Test credentials and state are destroyed after the test.

## Atomic multi-object snapshots (optional)

The native `cas::snapshot::SnapshotStore` implements an initial **byte-valued logical transaction**. It operates on a map of arbitrary named byte values, independent of append-only logs, application deployment, schema migrations or execution architecture. An app explicitly opts into this protocol; Skrynia itself remains a raw object store.

**Storage contract.** Each logical snapshot stores a mutable JSON head at `PREFIX.head` containing a format version, monotonically increasing revision, 32-character hexadecimal transaction ID and SHA-256 of one immutable manifest. The manifest at `PREFIX.manifest.TXID` maps logical member names to immutable blob keys `PREFIX.blob.TXID.NAME` and exact SHA-256 values. Unchanged blobs are referenced from previous manifests rather than copied. Names must follow Skrynia key syntax. Readers read the head once, then validate the manifest and all referenced blobs against the committed hashes. If objects are missing or altered, the reader **fails** rather than silently exposing partial contents.

**Commit algorithm.** `initialize(genesis_txid)` creates the empty manifest first, then create-only publishes the head; concurrent initialization converges on one validated head. For updates, `read()` yields an immutable in-memory `Snapshot` including a copy of its values and its original head digest. `commit(&old, txid, &desired_map)` stages all changed blobs and their immutable manifest with create-only operations, then performs exactly one byte-SHA256 CAS replacing the head. Successful CAS is the logical linearization point for the **whole map**. A stale head returns `Commit::Conflict`; the caller should re-read and *recompute* a proposed update rather than copying over a competing change. A repeated transaction ID with exactly the same staged bytes is safe; conflicting reused IDs are rejected.

A lost staging response is reconciled by reading that exact immutable object. A lost head CAS response yields `Committed` **only** when the current head exactly equals the proposed bytes; if it cannot establish that, the client reports an unknown outcome instead of replaying a transaction. A byte-identical replay against the now-current head also confirms `Committed`. Revision increments and unique transaction IDs prevent ordinary ABA within cooperating writers. The current API does **not** run user callbacks, auto-retry conflicts, coordinate external side effects, or create a cross-application transaction.

**Authority and retention.** Writers require `create` on the namespace and `modify` on its head; readers use public known-key GETs. Readers assume blobs and manifests are never deleted or modified by any authorized actor and that older immutable data is retained while it could be referenced. Applications must avoid issuing broad delete/modify permissions overlapping those immutable keys. Hash checks detect a violated assumption; they cannot stop an authorized attacker or restore missing data. Failed/stale transactions can leave immutable staged objects. **There is no garbage collector, pruning, bounded historical storage guarantee, or safe cleanup command**. Deletion needs a separately proven protocol covering in-flight writers and readers.

The [native example](../examples/snapshot.rs) demonstrates initialization, atomic two-member updates, stale-writer conflicts and repeated immutable transactions. The isolated [real HTTP integration test](../tests/ssh/protocols_live.sh) exercises Skrynia grants, CAS and blob reads with no delete authority. The public `values()` accessor is read-only so a caller cannot mutate an observed snapshot and trick unchanged-blob reuse.

This is an **initial** implementation with deterministic model fixtures, concurrent tests and real-server smoke coverage, not a claim of formal verification, machine-crash qualification or safe garbage collection.

## Garbage-collected append log

Never naively delete a currently acknowledged prefix: a stale writer can reuse the freed slot. Instead, establish disjoint epoch grants; revoke **all** create authority to an old epoch, observe its last committed contiguous slot after revocation, publish a sealed descriptor via CAS control, and only then activate the next epoch. The coordinator owns retention state. Registered consumers durably acknowledge positions; commit the retention floor under the same control as enrollment, and delete **only sealed epoch** entries below the floor. A reader must not infer its beginning from the smallest physical key during partial deletion.

Offline readers can block collection; dead coordinators can block rotation. Overlapping broad grants can defeat the seal. Unknown revocation outcomes and crashes between each step require recovery rules. Implement only after property/model tests validate revocation ordering, stalled readers, phantom slots, replays and malicious writers.

## Contributions, queues and notifications

An unordered event set can use unique create-only keys and scoped listing. Lexical key order is **not** chronological order. A task queue needs independently specified lease, claim, retry, ack and external side-effect rules; Skrynia itself guarantees no scheduler execution. Web Push messages are coalesced duplicate-prone wake-ups; read durable state to discover real changes.

## Validation levels

Distinguish: (1) enforced constraints on all authorized callers, (2) invariants assuming cooperating participants, and (3) accepted data proven by reader verification. For every high-level API publish which level each guarantee inhabits. Signatures can detect corrupted/unauthorized payloads according to a trust policy, but cannot stop an authorized holder from deleting data or refusing progress.


## Source-pinned real-server compatibility gate

GitHub CI also builds the real upstream Skrynia Rust server from commit `3139a82fe4a0292d75cc55c49f6a8bb9e6ff7b47` and runs `tests/ssh/protocols_live.sh` against an isolated temporary server/data root. The test issues narrowly scoped create/list and create/modify child capabilities, exercises the optional event set and multi-object CAS snapshots, and deletes all test state afterward. The upstream revision is deliberately pinned to avoid silently changing the tested protocol; update it explicitly with compatibility review. This does **not** test revocation of real grants, power failures, long-lived malicious tenants, or production operation.
