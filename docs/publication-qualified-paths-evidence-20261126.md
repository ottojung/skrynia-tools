# Qualified publication-path evidence (FK6Ag79l7WcLX7OJax3YvQ, 2026-11-26)

Disposable, development-only evidence for the real publication-root / provider /
nginx chain on the exact target bases. No production config, credentials, DNS,
deploy or service mutation was touched. The issue remains open.

## Exact inputs

| input | identity / SHA-256 |
|---|---|
| integration base | `37eea06ba7520ac4ea5d9f17e09e28f0d99a84c3` (`antonina/target-full-tools-integration-20261126`) |
| kernel | `db08cbc7496fafedc6a2e32df80395911b930879` release binary, reused byte-for-byte: `3da1852e1294f090bcee2c050f12e961e844ae5ae2d144b2453fd3cd2d54e89b` |
| provider | built from `publish-provider` at the base: `2de66285b15dd0cb5ef83b758cf12175a34743e46005e3218ecf8d3f4565c53f` |
| root WASM before fix | built from `publication-root` at the base: `66c7ec74159fa1e028937480dd61a4177d7fd6fbfda8a69d76a84487f70b6384` |
| root WASM after fix | built from the patched source: `40f4667a5df08969f14281cd4c1a22d571f007b6123f0c056169a4162667417f` |
| nginx | real `nginx/1.28.0` (`/gnu/store/db9fbd8i7dxn3k2rs7rll52d1gxx4wsy-nginx-1.28.0/sbin/nginx` locally; distro nginx in CI) |

The root WASM was rebuilt with Rust 1.93.0, `-Z build-std=std,panic_abort` and
the target's `--export-table` link flag. `check_wasm_abi.py` and
`check_target_abi.py` both report `OK` for the produced artifact.

## Actual failure and fix

Before the fix, the root's private-name check looked only at the first path
segment and listed unqualified names. The K10-qualified private paths
`user/publication-requests/00000001`, `user/security/program0.sha256`,
`user/programs/operator.cap`, `user/private/key`, `user/tokens/issuer` were
accepted (`code=0`): a ticket was written and the provider invoked, so a
qualified mailbox/private-root name could be mirrored into the nginx root.
`operator/...` paths were already rejected. After adding the `user` token to
the boundary set, all of these return `code=-1` before any ticket is written or
import invoked, while `apps/...` publications are unchanged. The provider's
generic source-disk policy and the separate user-root mailbox are untouched.

Directory semantics were re-checked through the same real stack: a missing
source directory returns `3` (INVALID_SOURCE) and creates no public node; an
existing non-symlink directory at the publication name returns `4`
(DEST_CONFLICT) and is left intact; an ancestor obstructed by a regular file
returns `4`; unpublish of an absent path is idempotent (`0`). The root's
create-if-absent mailbox `mkdir` succeeds whether the directory was missing or
already present, and fails closed with a mailbox error when a non-directory
obstructs it.

## Gates at the exact patched head

| gate | result |
|---|---|
| `real_wasm_chain.py` standard lifecycle | 21/21 checks `ok`, `ALL_OK=True`, rc 0 |
| `real_wasm_chain.py --overlap-total 300` | 27/27 checks `ok` (`overlap_gt_256_total_invocations total=300 completed=300`), `ALL_OK=True`, rc 0 |
| `slot_occupancy_model.py` | `ALL_OK=True` |
| `check_wasm_abi.py`, `check_target_abi.py` | `OK` |
| `cargo fmt --all -- --check` | rc 0 |
| `cargo test --locked --all-targets` | all targets 0 failed |
| `source_lints` self-tests | 7/7 |
| `cargo clippy --locked --all-targets -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity` | rc 0 |

No check was removed, skipped or relaxed: the existing negative path list,
provider-failure pass-through, 16-way concurrency, A-B-A active-pointer switch
and all overlap assertions remain, and the qualified-path and directory cases
were added on top. The integrated CI runs the standard lifecycle and the 300-call
overlap in one invocation and uploads the combined result file.

Rustdoc doctests were not run locally: this environment has no `rustdoc`
binary; `cargo test --locked --all-targets` (which excludes doctests) passed.
Clippy required `LD_LIBRARY_PATH` to point at the system GCC runtime because
this environment's loader does not find `libgcc_s.so.1` by default.
