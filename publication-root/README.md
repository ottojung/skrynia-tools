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
approved code and qualify it as a new artifact. Both roots use the operator's
private user-root disk namespace: this root writes only bounded ticket files under
`publication-requests/`, and the provider mounts that directory read-only. The
generic proxy never imports publication syscalls and never reads that directory.
Evaluated children receive only their delegated scoped filesystem authority, so
they cannot read, forge or delete tickets.

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
create-only exclusive semantics; an in-flight ticket is never reused. A
pre-invoke write or close failure removes only the ticket file this root just
created, never a pre-existing colliding file. A trap or an uncertain provider
outcome can still leave a ticket behind. This repository implements **no**
age-based janitor: an operator may reclaim such a file only after the provider
and every potentially in-flight request for that exact ticket are known to have
finished (for example in a controlled maintenance window), keyed on the exact
ticket name. Because tickets are never reused, reclaiming an old ticket cannot
affect any new request.

## Validation and authority

The root validates generic path grammar (bounded UTF-8, relative, no NUL,
newline, carriage return, backslash, absolute path, empty/`.`/`..` segments) and
rejects private roots (`security`, `programs`, `publication-requests`,
`operator`, `private`, `tokens`). App names, release identity, manifest hashes
and activation policy remain application/deployment concerns; this program
never encodes them.

## Release deployment gate

Release deployment must boot this generated root WASM through the kernel. A
deployment helper that POSTs directly to the HTTP provider is a convenience path
only; it does not satisfy the WASM publication lifecycle gate and must not be
recorded as its substitute. The downstream deployment owner wires the exact
compiled root artifact into the release flow and records its hash. Until that is
done and evidenced, the deployment gate is open (NOT RUN), regardless of this
program's source and CI status.

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
