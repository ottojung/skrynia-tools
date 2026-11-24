# Publication root program

A dedicated, generic root WASM for the operator-defined `publish` and
`unpublish` syscalls. It is **not** Program 0: the ordinary Program 0 remains a
token-checking proxy and does not import any syscall. Children are never given
publication authority.

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
O_EXCL semantics; an in-flight ticket is never reused. Stale request files from
trapped calls are reclaimed by an operator-side age-based janitor.

## Validation and authority

The root validates generic path grammar (bounded UTF-8, relative, no NUL,
newline, carriage return, backslash, absolute path, empty/`.`/`..` segments) and
rejects private roots (`security`, `programs`, `publication-requests`,
`operator`, `private`, `tokens`). App names, release identity, manifest hashes
and activation policy remain application/deployment concerns; this program
never encodes them.

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
