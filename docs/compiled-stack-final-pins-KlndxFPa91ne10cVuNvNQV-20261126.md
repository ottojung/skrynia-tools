# Target compiled-stack gates: exact tested artifact set

Host `marceline-dev` (hostname verified; Skrynia is host-eligible, no Viche
work). All digests below were re-derived from bytes with `sha256sum` on the
exact files used. No production, deploy, service mutation, merge, PR
promotion, push-to-main, live retry or capacity action.

## Scope and ownership

Only skrynia-tools tool-side fixtures/gates and their pins are changed here.
Kernel root selection (K10), P1 `cb_guest`/cap path source (T10, head
`07708b72`), the app P2 guest source (A3) and coordinator integration branches
were **not** edited; those delivered artifacts are consumed as-is.

## Exact source heads

- This branch: `antonina/t9-final-compiled-stack-KlndxFPa91ne10cVuNvNQV-20261126`,
  fast-forward from T9 `c4dd97d46520d12be75f29b980cae01e3036faef` verified by
  SSH `ls-remote` before work.
- Consumed artifacts: kernel source `db08cbc7496fafedc6a2e32df80395911b930879`
  (fresh clone), T10 tools `07708b725612fac4343641057c5877651f44a457`,
  A3 tools guest `feea7aae0c83574d45af330a80d08414bb650a33`.

## Artifact digest table (SHA-256 from bytes)

| artifact | sha256 |
| --- | --- |
| kernel, fresh release rebuild of `db08cbc` | `3da1852e1294f090bcee2c050f12e961e844ae5ae2d144b2453fd3cd2d54e89b` |
| kernel, earlier genuine `db08cbc` rebuild (cross-check) | `89fd1209d0d81ee6237049a27dc779a002e6d515f503ce458cc972fb6e8cb3b4` |
| kernel, K10 reference release (cross-check) | `0339ca954b40a5907936add3edde5e146e625599008899dec6f6c816f6d2a7f0` |
| P0, T10 qualified (`3f2d4032`) | `3f2d40321c2d2af8a77cbc210f863c963492f1b021885a88d963e8b5bd957357` |
| P0, tools `43f8927` qualified (`f173bf58`) | `f173bf58d012eeffded559790b285740897809caac1a9a01d43c90285491c327` |
| P1, final delivered T10 (`ec84e056`) | `ec84e056a209d4d58d990a231070c1b55f21f7b564a676426fdcb3599d94fc23` |
| P1, pre-final T10 (`9634e562`) | `9634e5624878bca85e6cbad3e1e2e0fd9cd92079180cf901c0516b148d773c39` |
| P2, A3 real guest (`19a59258`) | `19a592588d9f0987ec93b55837778aadf9673c6b35e814cacfa2f1d043e8d11a` |
| P2, tools example guest (`1bd197f4`) | `1bd197f45bde37e5454429df0f050f9fb43b7533e0de36ec2cedbe7c07613945` |
| scope probe (`4e1deee3`) | `4e1deee3680ed4bc66fa087a35bad975d1602e6b2a545d95125ed964c8b7eb21` |
| descriptor guest (`220d754c`) | `220d754cd38309681ae92d041efc2930d80211155e75502ef3b1e2291b423bd5` |

The fresh kernel embeds
`/workspace/t9-final-KlndxFPa91ne10cVuNvNQV-20261126/kernel-target/release/build/cranelift-codegen-*/out/*.rs`;
the two cross-check kernels embed the K10/T9 `db08cbc` build trees. No
`skrynia-k0-dingley`, `f538792`, `4ab7f3fc`, `020e14a` or `2fb719b` build path
or active pin was used.

## Four compiled-stack gates at the exact tested set

Artifact set: kernel `3da1852e…`, P0 `3f2d4032…`, P1 `ec84e056…`,
P2 `19a59258…`. Commands were the repository's own fixtures, with no
assertion skipped, weakened or faked. Logs: `gates/final-t10p0/*` in the
isolated workspace.

| gate | rc | key output |
| --- | --- | --- |
| `tests/v3_proxy/callback/real_scope.py` | 0 | `user-positive 17`; `stale-descriptor:read -2`; six read denials negative; `operator-rename -4`; `operator-directory -3`; `callback-eval -1`; `user-twin-read 9`; `user-twin-delete 0`; `PASS: compiled P0 callback isolates operator root, including after a trap` |
| `program0/integration/target_p0_ops.py` | 0 | `OPS5TO10_PASS HTTP/1.1 200 OK b'OPS5TO10_OK'`; `RAW 4f505335544f31305f4f4b` |
| `tests/v3_proxy/registry_recovery.py` | 0 | `PASS: SIGKILL after stage, restart, idempotent stage, seal and activate`; `PASS: filesystem obstruction (code 07), recovery and no stuck lock`; `PASS: second restart, stale CAS rejection and rollback` |
| `examples/theorem_lab_feedback_p2/tests/real_kit_p2.py` | 0 | `register-p2 200 ... kind ok`; `submit-e1-created` and every subsequent assertion PASS including `concurrent-no-lost-update`, `unauth-invoke-denied`, `victim-state-intact`, `lookup-after-restart`, `state-after-restart`; `ALL_OK=True` |

The same four gates were also run against the earlier genuine `db08cbc`
rebuild `89fd1209…` with the same P0/P1/P2 and were all rc=0
(`gates/combo-a3p2/*`), so the result is independent of one particular build
directory.

## Decisive negative combinations (no fake pass)

- P0 `f173bf58…` with the same kernel/P1/P2: gates 1 and 2 rc=0, gate 3
  `AssertionError: (15, 1, b'\x0b')` at `register`, gate 4
  `register-p2: HTTP/1.1 200 OK {'kind': 'err', 'code': 11}` and `ALL_OK=False`
  (`gates/alt-qualified-p0/*`). The T10 P0 is required by the final P1.
- Tools example P2 `1bd197f4…` with T10 P0/P1: gates 1-3 rc=0, gate 4 rc=1
  with `submit-e1 {'kind': 'ok', 'body': b''}` and `get-state-1 {'kind':
  'err', 'code': 8}` (`gates/prelim-t10p0/*`). The A3 real guest with a live
  callback context is required; this is an A3-owned P2 guest source item,
  not a kernel or P1 defect.

## Pin status

`examples/theorem_lab_feedback_p2/tests/real_kit_p2.py` now defaults to the
exact tested set above and fails closed on anything else. CI continues to
pass its own freshly built digests via `--*-sha256`; the coordinator must
reconcile the T10 tools head and the A3 tools guest into the integration lane
before CI reproduces this exact head. Whole release remains NO-GO until that
integration is done. @ottojung
