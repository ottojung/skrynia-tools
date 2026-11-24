# Exact-stack web-admin-v3 admin E2E evidence (tools `73d8fa1`, kernel `806c172`)

Source pins: tools `73d8fa12ad971d0a26eb1963c687d6de95054d89`, kernel
`806c172402880389eee94162a020c0f9c4c1bff2`, WASM rustc 1.93.0 release
cb-guest. Artifact kit `/workspace/skrynia-exact-kit-73d8fa1` (SHA256SUMS and
SHA256SUMS.release verified). **Release kernel used for all P1/P2 acceptance:**
`kernel-release` SHA256 `a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1`.
The dev/debug kernel is not used for acceptance (known 3.5 s compile-worker
deadline under unoptimized build).

## Exact artifact digests

| role | path (kit) | SHA-256 |
|---|---|---|
| kernel (release) | `kernel-release` | `a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1` |
| P0 (`root.wasm`) | `root.wasm` | `44e845a4ca1bfcf6713078ad40d2ce7d387595b08351a5a882bbc38549780c95` |
| P1 service | `service.wasm` | `c7239fc5bfd6977e7835eef461a75e274270d9e721d8a2d88121906b1b0c3a36` |
| echo guest | historical fixture | `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5` |
| trusted edge | existing gateway binary | `9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f` |
| P2 fixture | `theorem-p2.wasm` | `bc610533e52ecd9a5f4555adb6b31e9f83ee812ff96dd13ec0749124550624e0` |

## Results (all PASS, exact release kernel + kit WASM, no rebuild)

| harness | result | checks |
|---|---|---|
| `tests/web-admin-v3/real_native_e2e/p1_admin_ops.mjs` | PASS | 42/42 — real native opcodes 9/10/11/12/13, list pagination, sealed-ready, A->B->A CAS rev1->2->3, stale/ABA/unsealed/malformed/auth denials |
| `tests/web-admin-v3/real_native_e2e/p1_admin_adapter.mjs` | PASS | 23/23 — shipped adapter -> real P1 admin ABI, no AppPut/AppGet |
| `tests/web-admin-v3/real_native_e2e/run.mjs` | PASS | 36/36 — transport/forged-P0 framing/gateway auth boundary |
| `tests/web-admin-v3/real_native_e2e/fixtures/restart_reconcile.mjs` | PASS | 33/33 — drop-after-commit uncertainty via shipped Chromium UI, no blind repeat CAS (activateCount=1), explicit refresh to exact rev2, private state byte-identical across restart, post-restart CAS + conflict |
| `tests/web-admin-v3/real_native_e2e/chromium.mjs` | PASS | 25/25 — real Chromium trusted-origin UI, sealed identity, A-B-A rev1->2->3, stale/unsealed fail-closed, no secret leak, POST-only |

Machine-readable evidence written during the run (host-local, not committed):
`/tmp/opencode/exact-release-*.json`.

## Notes

- Drop-after-commit uses a loopback proxy that forwards ACTIVATE to the real
  trusted gateway -> kernel -> P0 -> P1, records the committed 42-byte reply,
  then returns an empty 200 to the browser. (A raw TCP reset was found to make
  Chromium retry the POST at the network layer, producing a false "blind
  repeat" signal; the empty-200 truncated reply avoids that artifact while
  still exercising the shipped adapter's uncertain-outcome path.)
- Private-state integrity is asserted by hashing every file under
  `user/programs/{active,blob,ready,appdata}` before and after restart.
- Chromium UI pagination is not explicitly exercised (PAGE_LIMIT=256); API
  pagination is covered by `p1_admin_ops.mjs` L3/L4.
- No production/deploy, no secrets, no runtime deadline changes, no duplicate
  kernel/guest build.
