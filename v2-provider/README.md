# Optional v2 API as an ordinary evaluated WASM program

This crate is an independently user-supplied nonzero WASM program,
not a named kernel Program 1. Skrynia loads only operator-selected
Program 0 for binary POST /. The skrynia-tools Program 0 checks its
private 256-bit operator bearer, removes that prefix, and generically
evaluates this v2 guest with two forward-only file descriptors and
a callback table. The kernel has no v2 routes or application logic.

The guest imports only skrynia.invoke/read/write, not privileged root
filesystem/RAM/mutation imports. Through the operator root's *generic*
callback, it can request open/read/write/close, fsync, rename, mkdir,
delete, directory iteration, random bytes, existence/is-directory,
nested generic eval, and the request-owned kernel mutation lock.
The v2 guest (not root or kernel) implements the v2 protocol.

## Transport and compatibility

Input/output are bounded JSON envelopes of the shared
v2compat::{Request,Response} types. The v2 method/path/query/headers
are data inside the outer authenticated v3 POST /. The guest handles
public GET/HEAD, ETags, ranges, OPTIONS and path validation.

A separate operator-only **loopback HTTP adapter** for legacy v2
clients is developed in skrynia-tools PR #41. Its operator bearer must
remain on the trusted host and NEVER appear in public browser scripts,
URLs, logs or public objects. Literal HTTP GET/HEAD/PUT routes are not
provided by the kernel. A secure public-facing reverse proxy requires
its own HTTPS/access controls; this guest and adapter are not yet
production qualified.

## Experimental grant-aware mutations

The ordinary guest additionally implements POST create, PUT replace
and DELETE on object paths, with user Bearer authorization, operation
and location scopes, SHA-256 content preconditions and ancestor quota
checks. The grant authority is loaded from operator-provisioned
JSON files under v2/state/grants/; exactly one root and a validated
delegation tree are required. A missing or corrupt authority fails
closed; this is **not** a complete on-demand issuer grant-management
API, and the operator must provision the root grant out of band.
Never place plaintext user/issuer bearer tokens in grant JSON:
the v2 code stores/compares SHA-256 digests.

Writes use the kernel's generic *request-owned* lock (PR #112) through
Program 0 callback opcodes 14/15, not a RAM-CAS spinlock. Under that
lock, the guest reads current content, validates preconditions,
loads the grant tree, checks affected quotas and applies the change.
Concurrent creates or stale-CAS replacements cannot both succeed.
After a trapped request, VM Drop releases the shared lock.

A replace writes an exclusively created, hidden same-directory
staging file; syncs, closes, atomically renames, then syncs the parent
directory. Precommit errors attempt to delete the staging file.
Delete syncs its parent directory. Creating nested directories syncs
their existing parents. The staging file prefix is hidden from the
object quota scan. A kernel-process restart confirms the resulting
file is readable; this is **not** a power-loss or fault-injection proof.

The object quota implementation currently recursively scans and
reads existing objects to compute sizes, which is neither scalable
nor bounded by the desired API complexity. Stale hidden staging
files after a hard process crash are not yet garbage-collected.
A parent-directory sync failure after a rename can leave an
unknown mutation outcome; clients MUST NOT retry automatically.

## Evidence and release blockers

Native unit tests use a fake generic callback. Explicitly run real
HTTP tests in tests/v3_proxy with independently compiled Rust Program 0,
the actual kernel from PRs #111/#112, and this separately compiled WASM.
They cover successful grant-authorized create/replace/delete, stale
hash denial, location/operation/token/quota failures, simultaneous
create and SHA-CAS races, restart persistence, and the existing
public-read/streaming/guest-isolation tests. The unprivileged WASM
must have exactly the invoke, read and write imports.

The JSON transport buffers at most 8 MiB inbound and 10 MiB encoded
outbound; legacy v2 supports larger objects. Full v2 issuer grant CRUD,
user-token issuance/revocation, correct quota accounting for all grant
metadata/subscriptions, Web Push, multi-process mutation coordination,
durability under every crash point, differential legacy parity,
production TLS/gateway access policy and data migration are NOT done.
The current program uses an operator-controlled data root and broad
generic root filesystem capabilities, so it is NOT a general safe
untrusted-plugin sandbox. All examples use disposable test data.
No production data or credentials are accessed.
