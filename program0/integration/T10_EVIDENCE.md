# T10 evidence — P1-wide userland lease, contention and trap recovery

Scope: issue `yLj3PJ10SraJWSte2B0qBw-overwhelm`, host `marceline-dev` (hostname
verified). Base is the reviewed tools integration `43f8927` (which already
contains T8 frame hardening `9573475c`, the random-op fix and qualified paths).
This branch continues the pushed recovery commits `b0b1526..86f2e88` after
verifying them against primary source; it is isolated from
`antonina/target-full-tools-integration-20261126`. No merge, PR promotion,
deploy or production change. Issue stays OPEN.

## Implementation

Program 0 relays ops `12/13` over **one absolute shared RAM byte** at
`alloc::LOCK_AT = 80`, not `c.base + LOCK_AT` (which was unique per slot). The
byte is an owner tag: `0` free, `1..=16` = slot index + 1. Acquisition is
`CAS(0 -> owner)`; release is owner-only `CAS(owner -> 0)`; every request exit,
including a trapped child, runs the same owner-only `reclaim`. Replies keep the
P1 boolean convention (`1` acquired / `0` contended for op 12, `0` released /
`-4` non-owner for op 13). This is a generic userland resource lease in shared
RAM: no kernel lock, no native metering, no app policy in P0. Nested contexts
inherit the parent slot base, so an owner tag cannot be remapped mid-request.
T8's exact-frame check is retained.

The P1 adapter uses `user/programs` as its registry base (K10 namespace
tokens) and treats **only `E_NOENT = -8`** as absence in `disk_read`, `disk_ls`
and `sync_dir`; EIO (`-5`), EBADF (`-2`), EINVAL (`-3`), permission (`-4`) and
unsupported remain hard errors. A host test asserts that classification.

## Exact artifacts (all pinned by SHA-256 at run time)

| Artifact | Identity |
|---|---|
| tools code/artifact head | `41a0d16b5ac4b80ae0278e8703829f436fcc65f9` (final branch head reported on the board) |
| kernel K10 head | `db08cbc7496fafedc6a2e32df80395911b930879` |
| frozen `skrynia-server` binary | `1d07c08f2392b32b6fa2b3b079ef25cc2c20b0e3f8484ccb0b75f124ea7ab81c` |
| fixed P0 WASM (rebuilt from this head) | `3f2d40321c2d2af8a77cbc210f863c963492f1b021885a88d963e8b5bd957357` |
| pre-fix P0 WASM (`43f8927`) | `f173bf58d012eeffded559790b285740897809caac1a9a01d43c90285491c327` |
| rebuilt P1 WASM (`cb-guest`, `opt-level=s`, stripped, `--export-table`) | `ec84e056a209d4d58d990a231070c1b55f21f7b564a676426fdcb3599d94fc23` |
| `t10_lock_guest.wasm` (rebuilt from this head) | `c93956f2f8782ba7bf3ba2b713909c357b522f841bbf67051f1d2486a806716c` |
| P1 guest surface | imports exactly `[('p0','invoke',0)]`, exports `_start`, zero `skrynia.*` imports (`TARGET_GUEST_SURFACE_PASS`) |

The kernel binary is a disposable rebuild whose SHA changed while other lanes
were active; a copy was frozen in `/tmp/opencode/t10-oef1-artifacts/` and all
runs below pin that exact SHA. P0 and the guest were rebuilt from this exact
head (1.93.0 toolchain, wasm32-unknown-unknown); their SHAs reproduce exactly.

## Deterministic real-kernel contention and trap recovery

`python3 program0/integration/t10_lock_http.py --kernel K --p0 P0 --guest G
--workdir W --kernel-sha256 ... --p0-sha256 ...` boots the compiled K10 kernel
with a fake bearer and disposable temp roots. Two real HTTP requests overlap:

* holder acquires the lease, publishes `held.bin`;
* contender publishes `ready.bin`, waits for a harness-written `go.bin`, then
  performs its CAS and publishes its verdict to `result.bin`;
* holder releases only after `result.bin` exists.

`ready -> go -> CAS -> result` is a two-way barrier, so the contender CAS is
inside the holder lease by construction, not by CPU timing. A fresh server
tests trap recovery: a guest acquires, marks `held.bin` and traps; a later
request must then acquire and release cleanly.

Results (each run pinned to the artifacts above):

* Fixed P0, two independent runs: `ALL_OK=True` — `holder-held-and-released`
  (`LOCK_HELD_RELEASED`), `contender-excluded` (`LOCK_CONTENDED`),
  `trapping-holder-fails`, `trap-lease-recovered` (`LOCK_RECOVERED`).
* Pre-fix per-slot P0 (`43f8927`), same fixture, two runs: `ALL_OK=False` —
  `holder-held-and-released=FAIL` (`LOCK_ACQUIRE_FAIL`: successful `0->1` CAS
  observed byte `0` was relayed as "not acquired"),
  `contender-excluded=FAIL` (`NO_HOLDER_MARKER`),
  `trap-lease-recovered=FAIL` (`LOCK_ERROR`). This is the red control.

Correction to the earlier unchecked evidence: the pushed `86f2e88` fixture
looked green with guest SHA `05b261b5…`, but that guest never drained the
WRITE/SYNC/CLOSE replies in `write_file`, so the next EXISTS read a stale reply
code and the holder released early. Re-running it against the current kernel
gave `LOCK_ACQUIRED` (vacuous). Draining every reply and adding the explicit
barrier fixed the fixture; the green/red results above are non-vacuous and
stable across runs and kernel rebuilds.

## P1 read/write protocol audit

`dumb_p0_http.py` (frozen kernel, fixed P0, rebuilt P1, real echo guest,
SHA-pinned): PASS for `register`, `lookup`, `appdata-write-read`,
`appdata-large-put`, `appdata-large-get-exact` (256,605 bytes), `appdata-cas`,
`register-sibling`, `sibling-appdata-isolated`, `lookup-after-restart`,
`appdata-after-restart`, `wrong-operator-cap-denied` (code 9),
`wrong-bearer-denied`, `missing-bearer-denied`, `shadow-user-verifier-denied`.
`invoke-p1-p2` still returns an empty body: that is the coordinator-owned P2
callback-adapter gap (hardcoded `AppGet("cb")` on this base), unrelated to
lease or root handling. No absence is inferred from EIO/EBADF; `mkdir_p`'s
pre-existing `EINVAL` tolerance and the P2 adapter gap are recorded as residual
items outside this issue's ownership.

## Gates

At the exact tested head: `cargo fmt --all -- --check` rc=0; source-lint
self-tests 7/7; `cargo test --locked --offline --workspace --all-targets`
rc=0, 246 passed / 0 failed across 17 suites (including
`lease_byte_is_shared_and_owner_only`,
`framed_ops_exclude_other_requests_without_releasing_their_lease`,
`request_exit_reclaims_a_trapped_owner_lease`,
`only_explicit_enoent_means_absence`); strict Clippy
(`-D correctness/suspicious/perf/complexity`) rc=0. Clippy needed
`LD_LIBRARY_PATH=/gnu/store/5n0p14imxxnygk39wfx526smdfv7qh94-gcc-15.2.0-lib/lib`
for a reused build-script binary in this host environment; that is an
environment library path, not a source change.

## Claim boundary

This proves userland cross-request exclusion, owner-only release and trap
recovery on a disposable real kernel with exact artifacts. It is not Theorem
Lab end-to-end acceptance, a browser/provider run, a packaged P1 module claim,
a K10/kernel change, a merge, a production cutover or an old-ABI qualification.
The issue remains OPEN.
