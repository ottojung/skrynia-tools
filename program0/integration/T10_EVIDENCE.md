# T10 recovery evidence — K10 real-kernel lease and P1 persistence

Scope: the T10 issue (`yLj3PJ10SraJWSte2B0qBw-overwhelm`) recovery after host
interruption. Base is tools integration `43f8927`; T8 frame hardening, the
random response fix and the qualified private-root paths are retained. No main
merge, PR promotion, deploy or production change.

## Exact inputs

| Artifact | Identity |
|---|---|
| tools branch | `antonina/t10-p1-wide-lock-recovery-20261126` |
| tools base / tested head | `43f8927` / `f12e9a87e56ea8cf23a2e0660a43c76c2e9b346f` |
| kernel K10 head | `db08cbc7496fafedc6a2e32df80395911b930879` |
| `skrynia-server` (release) | sha256 `0339ca954b40a5907936add3edde5e146e625599008899dec6f6c816f6d2a7f0` |
| fixed P0 wasm | sha256 `3f2d40321c2d2af8a77cbc210f863c963492f1b021885a88d963e8b5bd957357` |
| pre-fix P0 wasm (43f8927) | sha256 `f173bf58d012eeffded559790b285740897809caac1a9a01d43c90285491c327` |
| rebuilt P1 wasm (`cb-guest`, stripped) | sha256 `ec84e056a209d4d58d990a231070c1b55f21f7b564a676426fdcb3599d94fc23` |
| `t10_lock_guest.wasm` | sha256 `05b261b53412d4106c4d73ed07fead57641f2ef838b2148c84632895498fe78e` |

P0/P1 were rebuilt from the exact branch head with the 1.93.0 toolchain and
`-C link-arg=--export-table -C strip=symbols -C opt-level=s -C panic=abort` for
P1 (the unstripped build exceeded P0's 2 MiB staging cap).

## Deterministic real-kernel lease test

`python3 program0/integration/t10_lock_http.py --kernel K --p0 P0
--guest t10_lock_guest.wasm --workdir W`. Two HTTP requests are forced to
overlap: the holder acquires the lease and publishes a `held.bin` marker; the
contender then publishes its CAS verdict to `result.bin`; only after that
result exists may the holder release. The contender's CAS therefore runs
inside the holder's lease. A fresh server handles the trap case: a guest
acquires the lease and traps; a later request tries to acquire it again.

Fixed P0, three independent runs: `ALL_OK=True` each time —
`holder-held-and-released`, `contender-excluded` (`LOCK_CONTENDED`),
`trapping-holder-fails`, `trap-lease-recovered` (`LOCK_RECOVERED`).

Pre-fix P0 `43f8927`, same fixture: `ALL_OK=False` —
`holder-held-and-released=FAIL` (`LOCK_ACQUIRE_FAIL`: successful CAS observed
byte `0` was read as "not acquired"), `contender-excluded=FAIL`
(`NO_HOLDER_MARKER`), `trap-lease-recovered=FAIL` (`LOCK_ERROR`). This is the
red control: the old relay neither reported acquisition correctly nor
excluded a second request on the shared resource.

## P1 persistence on the qualified-root kernel

`python3 program0/integration/dumb_p0_http.py` with the rebuilt P1 produced
PASS for `register`, `lookup`, `appdata-write-read`, `appdata-large-put`,
`appdata-large-get-exact` (256,605 bytes), `appdata-cas`,
`register-sibling`, `lookup-after-restart`, `appdata-after-restart` and all
negative cases. `invoke-p1-p2` returned an empty body: this base still has the
known P2 callback-adapter gap described by the coordinator (hardcoded `AppGet`
rather than complete opaque CB frames), unrelated to root or lease handling.
No production data was touched; temp dirs are disposable.

## Gates

`cargo fmt --all -- --check`, source-lint self-tests (7/7), strict Clippy
(`-D correctness/suspicious/perf/complexity`) and `cargo test --locked
--all-targets` pass at the tested head. The build-time Ivanka source lints run
through `build.rs` on every crate tree, including `program0/`.
