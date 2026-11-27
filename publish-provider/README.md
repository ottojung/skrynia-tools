# skrynia-publish-provider

A small, independent HTTP provider for the operator-defined `publish` and
`unpublish` syscalls. It is **not** Skrynia kernel code, not Program 0, not a
CLI, and not an administrative tool. It has no command-line parser; all
configuration comes from the environment.

## Trust boundary

The kernel sees only a numeric ticket. Path data crosses the boundary through
an **operator-private mailbox**, never through the kernel RAM or a public HTTP
parameter. The provider binds a loopback address only and exposes exactly one GET discovery endpoint and two POST operation endpoints. It never serves source bytes and never reads kernel memory.

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
All three roots (requests/mailbox, source disk and nginx publication) must be
disjoint in **both** directions, enforced at startup: a mailbox nested under the
source root could otherwise be published by a generic relative path. The
publication VM boots its own user root containing only the mailbox; the source
artifact disk is a separate read-only root. The nginx publication
root must be owned exclusively by this provider; external writers can race the
inspect/remove step and cause fail-closed conflicts. Published source ancestors
must remain trusted/immutable for the publication lifetime, because symlink
resolution follows the live absolute source path.

## Syscall ABI

The target Skrynia kernel posts to `<SKRYNIA_SYSCALL_URL>/<name>`
with a typed integer wire body. Both operations have exactly one i32
argument and one i32 result; all 32 bits are preserved as unsigned tickets
or result codes. The body is seven bytes: version `0x01`, count `0x01`,
type tag `0x32`, then four little-endian bytes. A successful response is
HTTP 200 with the same layout. Invalid version, count, type, length or
trailing bytes receive HTTP 400 before any publication effect. Unknown
function routes receive HTTP 404. The old four-byte wire is not accepted.

Set `SKRYNIA_SYSCALL_URL` to the provider origin with no path prefix.
`GET /syscalls_list` returns ordered type metadata:

```json
{"functions":[{"name":"publish","params":["i32"],"results":["i32"]},{"name":"unpublish","params":["i32"],"results":["i32"]}]}
```

The provider does not advertise copy or require access to caller RAM or
descriptors. Provider POST /version is optional; this provider uses the
discovery endpoint alone.

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

This HTTP service defines the publication syscalls' operational behavior. A
custom executable using `libskrynia` may eliminate the HTTP transport only
by reproducing this complete discovery list and the same results and effects.
There is no separate class of syscall names or precedence rule.

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
and never the private `publication-requests` directory itself. The provider is generic over valid relative file paths and does not hardcode an
application namespace; the migration/root writer enforces application-level
authorization and public/private policy. The source root must be disjoint from
mailbox/operator secrets, and the provider never serves private trees. All traversal uses pre-opened root descriptors
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

The Rust tests spawn the real binary over loopback HTTP and cover `GET /syscalls_list`,
published symlink targets, idempotent repetition, concurrent tickets, missing
tickets, hostile path grammars, symlinked sources, conflicting real files,
wrong symlink targets and symlinked destination ancestors. The container smoke
test exercises the same ABI against the built image with read-only source and
mailbox mounts.

### Disposable target kernel transport smoke test

Build the target Skrynia kernel and this provider independently, then run
`KERNEL_BIN=/absolute/path/to/skrynia-server PROVIDER_BIN=/absolute/path/to/skrynia-publish-provider python3 publish-provider/target_kernel_smoke.py`
from the tools repository root. The test uses temporary roots, loopback ports,
and a scalar-only WASM fixture. It verifies typed discovery, the kernel
version metadata, and actual idempotent publish/unpublish effects. It does
not qualify the publication-root WASM, Program 0, Program 1, apps or production.
