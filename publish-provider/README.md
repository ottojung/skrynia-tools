# skrynia-publish-provider

A small, independent HTTP provider for the operator-defined `publish` and
`unpublish` syscalls. It is **not** Skrynia kernel code, not Program 0, not a
CLI, and not an administrative tool. It has no command-line parser; all
configuration comes from the environment.

## Trust boundary

The kernel sees only a numeric ticket. Path data crosses the boundary through
an **operator-private mailbox**, never through the kernel RAM or a public HTTP
parameter. The provider binds a loopback address only and exposes exactly three
POST endpoints. It never serves source bytes and never reads kernel memory.

There are three disjoint roots, mounted separately:

| Environment variable | Provider access | Purpose |
| --- | --- | --- |
| `PATH_TO_PUBLICATION_REQUESTS` | read-only | Operator-private mailbox of ticket files |
| `PATH_TO_SKRYNIA_DISK` | read-only | Source regular files (private operator disk) |
| `PATH_TO_NGINX_SKRYNIA_ROOT` | read/write | Publication symlinks served by nginx |

The same relative path is the key in both the source and publication roots.
`publish` resolves `PATH_TO_SKRYNIA_DISK/<rel>`, creates missing parent
directories under `PATH_TO_NGINX_SKRYNIA_ROOT`, then creates a symlink whose
**absolute target** is the corresponding source file. `unpublish` removes only
that matching symlink; it never unlinks the source, a regular file, or a
symlink with a different target.

The source root must be mounted at the **identical absolute path** inside the
nginx container, because the symlink target is an absolute path. nginx needs
read + execute permission on every source ancestor and on the publication
directory, and symlink following must be enabled (`disable_symlinks off`).
The mailbox root must never be mounted into nginx or any public document root.
Public and private roots must not overlap in either direction, so nginx cannot
serve the mailbox or private disk through a prefix alias. The nginx publication
root must be owned exclusively by this provider; external writers can race the
inspect/remove step and cause fail-closed conflicts. Published source ancestors
must remain trusted/immutable for the publication lifetime, because symlink
resolution follows the live absolute source path.

## Syscall ABI

The kernel's HTTP fallback (Skrynia main, `rust-server/src/sys/extensions/http`)
posts to `<SKRYNIA_SYSCALL_URL>/<name>` with `Content-Type:
application/octet-stream` and a body of exactly four little-endian bytes. A
successful response is HTTP 200 with exactly four little-endian result bytes.
The provider accepts no other body length and emits no other response length.

Configure the kernel with `SKRYNIA_SYSCALL_URL` set to the provider origin with
no path prefix. The kernel performs `GET <base>/syscalls_list` with no request
body and expects HTTP 200 with a plain JSON array of strings, for example
`["publish","unpublish"]`. Per-syscall invocation is unchanged:
`POST <base>/<name>` with four little-endian bytes in and out.

Result codes (u32 bit patterns):

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | missing, non-regular or symlinked ticket file |
| 2 | invalid path bytes/grammar |
| 3 | invalid source: absent, non-regular, or symlinked source/ancestor |
| 4 | destination conflict: real file, wrong symlink target, or symlinked ancestor |
| 5 | I/O error |
| 6 | internal invariant failure |

## Mailbox contract

The writer (the WASM userland bridge) allocates a fresh u32 ticket without
overwriting concurrent requests, creates one file named by exactly eight
lowercase hex digits of the ticket in `PATH_TO_PUBLICATION_REQUESTS`, writes
the exact UTF-8 relative path bytes with no trailing newline, invokes the
named import, then removes its request file. Providers only read these files;
the result returns in the HTTP body, so there is no result file and no
provider write to the mailbox.

The path grammar matches the kernel's `clean_relative` plus the mailbox
contract: non-empty UTF-8, no NUL/backslash/newline, no `//`, no trailing `/`,
no absolute path, no `..` segment, `.` segments ignored, at most 4096 bytes,
and never the private `publication-requests` directory itself. The provider also requires the first path component to be `apps/`, so a hostile
or malformed ticket cannot publish private operator files outside the public app
namespace. Finer-grained authorization remains the root writer's responsibility. All traversal uses pre-opened root descriptors
with `O_NOFOLLOW` on every component. Publish and unpublish calls are
serialized in-process; repeated identical calls are idempotent.

## Testing and container

```sh
cargo test --locked --all-targets --manifest-path publish-provider/Cargo.toml
cargo clippy --locked --all-targets --manifest-path publish-provider/Cargo.toml -- -D warnings
cargo fmt --manifest-path publish-provider/Cargo.toml --all -- --check
podman build -f publish-provider/Dockerfile -t skrynia-publish-provider:test .
CONTAINER_ENGINE=podman bash publish-provider/container_smoke.sh skrynia-publish-provider:test
```

The Rust tests spawn the real binary over loopback HTTP and cover `/version`,
published symlink targets, idempotent repetition, concurrent tickets, missing
tickets, hostile path grammars, symlinked sources, conflicting real files,
wrong symlink targets and symlinked destination ancestors. The container smoke
test exercises the same ABI against the built image with read-only source and
mailbox mounts.
