# Publication root program

A dedicated, generic root WASM for the operator-defined `publish` and
`unpublish` syscalls. In the kernel sense this program **is** the operator-supplied
root program ("Program 0" is the kernel's name for the single root it boots): the
kernel links `syscall.*` imports only into that root. It is a *different
deployment* from the optional generic token-checking proxy example in this
repository, which is an ordinary root that imports no syscalls and only relays a
private 256-bit token. Both are operator-chosen roots, never evaluated children;
children are never given publication authority.

## Relation to the token-proxy service

Only one root WASM is active per kernel instance. A publication deployment boots
this root; a pure HTTP-relay deployment may boot the generic token proxy instead.
The two are not composed inside an evaluated child. If one deployment needs both
behaviours, the operator must build a single qualified root WASM containing the
approved code and qualify it as a new artifact.

Storage is separated by deployment. The ordinary token proxy delegates its whole
user root to its trusted persistent service. This publication root instead boots
with its **own** kernel user root and mailbox: it writes bounded ticket files
under `publication-requests/` on that root, and the provider mounts only that
directory read-only. The source artifact disk is a **different** read-only root;
the provider enforces disjoint requests/source/nginx roots in both directions at
startup so a relative path cannot expose the mailbox. The generic proxy never
imports publication syscalls and does not run in a publication deployment.
Evaluated children receive only their delegated scoped filesystem authority, and
the kernel instance has no mailbox authority outside this root.

## Why a separate root

The kernel links `syscall.*` imports only into Program 0 and rejects them in
evaluated children. During a mediated callback the kernel replaces the root's
filesystem authority with the child's delegated scope, so a callback cannot
safely own a private mailbox. This program therefore runs as the root for the
publication lifecycle and performs the whole operation synchronously in its
main entry point. There is no queue, no late effect and no fake result.

## Request and response

Request body to `POST /`:

```text
[version=0x01][32-byte operator token][op: 1 publish | 2 unpublish][u32 LE path length][exact UTF-8 path bytes]
```

Response body is one little-endian i32:

| Code | Meaning |
|---|---|
| 0 | provider confirmed success |
| 1..6 | provider failure (see provider ABI) |
| -1 | invalid/missing authentication, frame or path |
| -2 | mailbox allocation/write failure; no import was invoked |
| -3 | unknown outcome; the import may have taken effect |
| -4 | confirmed result but request-file cleanup failed |

A trapping syscall produces no response at all; the caller must treat that as
unknown, never as success. Tickets are fresh random u32 values created with
create-only exclusive semantics, so concurrent requests cannot overwrite an
existing ticket file while it exists. This does not make ticket numbers globally
unique across time: a random u32 is only collision-protected by the currently
existing file. A pre-invoke write or close failure removes only the ticket file
this root just created, never a pre-existing colliding file. A trap or an
uncertain provider outcome can still leave a ticket behind. This repository
implements **no** age-based janitor or automatic reclamation. An operator may
reclaim such a file only in a quiesced maintenance window after the provider and
every potentially in-flight request for that exact ticket are known to have
finished, keyed on the exact ticket name. Never reclaim by age alone while
requests could still be in flight.

## Validation and authority

The root validates generic path grammar (bounded UTF-8, relative, no NUL,
newline, carriage return, backslash, absolute path, empty/`.`/`..` segments) and
rejects private roots (`security`, `programs`, `publication-requests`,
`operator`, `private`, `tokens`). App names, release identity, manifest hashes
and activation policy remain application/deployment concerns; this program
never encodes them.

## Target ABI port

All `skrynia` fixed imports now use the owner-approved target ABI: byte-only
`read1`/`write1`, signed relative `seek`, `truncate`, single-byte `cas1` and
u64 shared-RAM addresses/lengths for `open_disk`/`mkdir`/`delete`/
`secure_random_bits`. No caller-linear-memory pointer or bulk `read`/`write`
crosses the host boundary. The root exports exactly `skrynia_main(i32,i32)->i32`
plus `memory`; `publish`/`unpublish` keep the typed `(i32)->i32` provider
signature in the root-only `syscall` namespace.

Shared RAM is one unpartitioned byte region shared by every concurrent request,
so staging cannot use a per-process fixed offset. Bytes 0..255 are one CAS
occupancy byte per slot; a request claims a free byte with a sequentially
consistent `cas1` (0 -> 1) and releases it on drop, so no global RAM mutex is
needed and a wrapped counter cannot reuse a live slot. Each invocation then
uses its own bounded 8 KiB slot for the verifier path, mailbox path, random
ticket and request path; a per-slot bump cursor keeps those regions disjoint
and `seek` results are checked against the requested offset. The RAM is sized
with `truncate` only when still empty, so no request resizes a live backing
during normal operation. A trapping request can leave its occupancy byte set;
like a trapped mailbox ticket, that is reclaimed only in a quiesced window.

`tests/check_target_abi.py` parses the compiled WASM and asserts the exact
target signatures and the preserved provider signature. The disposable
`tests/real_wasm_chain.py` boots the real kernel and real provider, drives the
compiled root and checks root-token, path/security, provider-failure and
16-way concurrent publication behaviour. The provider HTTP codec is owned and
built by the coordinator; this crate never edits it.

## Release deployment gate

Release deployment must boot this generated root WASM through the kernel. A
deployment helper that POSTs directly to the HTTP provider is a diagnostic
convenience path only; it does not satisfy the WASM publication lifecycle gate
and must not be recorded as its substitute.

The downstream WASM-root gate is now qualified in disposable CI, not "NOT RUN":

- Apps PR #342 merged as `9e1781ec85e8ed4f15ad2b5b30f9a433a475b81e`,
  integrating the real WASM-root publication lifecycle and bounded activation
  (39/39 local lifecycle checks, 31/31 unit checks, exact-head CI green).
- Apps PR #350 head `d6fd2d50038f89f81a96eae07d33cdd307256943` merged as
  `735d186bd3b2ae8abe057f7b9677a8182a82d263` with 5/5 exact-head check-runs,
  including the `static-publication` workflow: 31/31 unit checks and a real
  lifecycle `ALL_OK`, with pinned source identities and the served publication
  contract recorded.

That downstream evidence means the real lifecycle has been exercised; it is not
production authorization. Production DNS/TLS, credentials, application data
migration and cutover remain separate operator gates, documented downstream in
`docs/static-host-v3-deployment.md` and
`qualification/publication-isolation/README.md` in `skrynia-apps`.

## Build and verify

```sh
cargo build --locked --release --target wasm32-unknown-unknown -p skrynia-publication-root
python3 publication-root/tests/check_wasm_abi.py \
  --root target/wasm32-unknown-unknown/release/skrynia_publication_root.wasm \
  --generic-p0 target/wasm32-unknown-unknown/release/skrynia_program0.wasm
```

The checker asserts that the produced root imports both syscalls with
`i32 -> i32`, exports `skrynia_main`, has no foreign import namespace, and that
generic Program 0 imports no syscalls.

The real lifecycle test drives the compiled kernel, this WASM, the real
provider and real nginx:

```sh
python3 publication-root/tests/real_wasm_chain.py \
  --kernel KERNEL --provider PROVIDER --root ROOT_WASM \
  --results /tmp/opencode/wasm-publication-results.json
```
