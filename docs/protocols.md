# Optional client protocols

The ordinary native service offers capability-authorized byte objects and
digest-conditioned mutations. None of the following protocols is mandatory
for deploying an application.

| Module | Contract |
| --- | --- |
| Native data client | Binary operations 6 (put), 7 (read), 8 (delete) |
| CasCell | Read bytes, compute digest, conditional replace |
| AppendLog | Contiguous slots, idempotent cooperating writers |
| SnapshotStore | Immutable manifests/blobs, one CAS head |
| EventSet | Generic unordered contributions over an injected enumerating store |
| Subscription status | Read-only operation 27, exact subscriber-scoped status |

The native client does not implement object enumeration or create-only
namespace credentials. EventSet therefore requires a caller-provided
EventStore implementation; do not pretend native listing is available.

## Native wire format

Requests start with version 1 and an operation byte. The capability is a
u16 little-endian length (32) followed by 32 bytes. A key is a u32
little-endian length followed by key bytes. Put includes an absent/present
precondition, a digest for present, then a u32-length-prefixed value.
Delete includes the expected digest. Read contains just the capability and key.

The response is version 1, status 0 or 1, and a u32 little-endian body length
followed by exactly that many bytes. Error 8 identifies a missing object;
error 10 is a structured conflict. Successful put returns the content digest.
Read returns the value; successful delete returns an empty body. The SDK
checks the complete frame and applies a 16 MiB frame bound. Other errors or
uncertain transport outcomes are surfaced to the caller.

## Subscription status (ordinary P1, OP27)

OP27 is a narrow, read-only status check in the ordinary nonzero service. It
never exposes endpoint, p256dh or auth material to the caller and grants no
sender or provider authority. A caller supplies the existing narrow
`Kind::Subscribe` grant capability, the exact object key, the exact subscriber
bytes, and the 32-byte SHA-256 of the opaque subscription bytes it submitted
to OP16. The service hashes the stored subscription with the same canonical
bytes, verifies the durable grant subject and generation, and compares digests
in constant time. No durable state is written; the mutation lock is taken for
a consistent snapshot and lock contention is reported explicitly.

Request bytes after version 1 and opcode 27:

| Field | Encoding |
| --- | --- |
| grant capability | u16 LE length 32, then 32 raw capability bytes |
| exact key | u32 LE length, then key bytes |
| subscriber | u32 LE length, then 1..256 raw subscriber bytes |
| expected subscription hash | u16 LE length 32, then 32 raw SHA-256 bytes |

No trailing bytes are accepted; truncated frames are rejected.

Response (version 1, status byte, u32 LE body length):

| Status | Body | Meaning |
| --- | --- | --- |
| 0 | `[0x01]` + 32-byte stored subscription hash | Active: grant, subject, generation and exact hash all match |
| 0 | `[0x00]` | Absent: no stored subscription for this exact subscriber |
| 1 | `[0x08]` | Mismatch: authorized record exists but hash differs |
| 1 | `[0x09]` | Revoked or invalid grant: missing grant, wrong key/subject, rotated or mismatched generation, or corrupt record |
| 1 | `[0x0B]` | Busy: mutation lock unavailable, no state change |
| 1 | `[0x07]` | IO error, including lock release failure |
| 1 | `[0x01]` | Malformed request |

This operation reports stored subscription state only. It does not send
notifications, contact a push provider, verify delivery, or expose any
provider credential. Provider delivery remains a separate isolated sender
concern and is not implied by an Active response.

## CAS cell

A pure update callback can be reevaluated after a definitive conflict.
Never reevaluate it automatically after an uncertain mutation outcome.
Use a monotone revision and unique transaction identity when the application
needs to distinguish A-B-A even when payload values repeat.

## Append-only log

The log uses epoch e000001 and twelve-digit sequential keys. Cooperating
writers race to create the first missing consecutive slot. If another writer
wins, read the slot to distinguish the same operation ID and identical bytes
from a different operation or a collision. Reconcile an uncertain create with
the exact slot; uncertain absence is not permission to replay it. Readers stop
at the first missing slot, which is not necessarily a permanent end.

This requires fresh observations, no modification or deletion of existing
slots, and cooperating writers. A native namespace write capability does not
enforce these restrictions. A malicious writer can disrupt progress. A scan
from genesis is not constant-time.

## Unordered event set

The generic EventSet uses PREFIX.ID, with a 32-character lowercase hexadecimal
ID, and reconciles duplicate/uncertain creates by comparing exact bytes.
Its injected EventStore must supply bounded ordered pages. Scans validate
ordering and advancing cursors, but are not consistent snapshots: inserts
before an already observed cursor can be missed. A completed scan only means
the observed pagination ended. This abstraction does not supply a native
index, guarantee retention, or impose an application schema.

## Atomic multi-object snapshots

SnapshotStore maintains a mutable PREFIX.head containing the revision,
transaction identity and digest of an immutable manifest. A manifest refers
to immutable blob keys and expected content hashes. Readers validate all
referenced objects. Missing or changed data fails closed.

Initialize the manifest before creating the head. Stage changed blobs and
a new manifest before conditional replacement of the head. The single CAS
is the logical publication point for the entire map. On conflict reread and
recompute; on an uncertain result compare the authoritative head with the
intended bytes. Unique transaction identity prevents a false match across
ordinary A-B-A changes. This is a cooperative protocol, not a multi-key
transaction enforced by the service.

Writers and readers must preserve referenced immutable data. A writer with
namespace-wide authority could break the protocol. Abandoned staging objects
can accumulate; no collector or bounded storage guarantee is supplied.
Deletion needs a separately proven protocol covering active readers and
writers. See [the native example](../examples/snapshot.rs).

## Validation

Native model tests include concurrent writers, lost responses, malformed
objects and a long state-machine trace. The wire compatibility test uses
the real service decoder. Browser tests exercise disposable wire fixtures.
Full compiled-stack acceptance is a separate gate.
