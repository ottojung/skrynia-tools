# Pinned native P1 admin-opcode acceptance (real gateway + kernel + P0 + P1)

`p1_admin_ops.mjs` is a permanent, non-vacuous Node test for the **dedicated
ordinary-P1 admin opcodes**. It is a test only — no CLI, no simulator, no mock
backend, no injected fake capabilities, no Cargo/Podman/WASM rebuild, no new
`node_modules`. It reuses the bootstrap/pin helpers already exported by
`run.mjs` and the shipped `createBinaryPostTransport()` adapter.

```
adapter.post(opaque bytes)  ->  trusted edge gateway (prepends 0x01||token[32])
  ->  kernel POST /  ->  dumb P0 (auth + 33-byte strip, decodes no opcodes)
  ->  ordinary WASM P1 (opcodes 9/10/11/12/13 enforced in compiled code)
```

The adapter defines no opcodes; P0 decodes none. If the compiled P1 did not
implement the admin API, every positive check below would fail. Nothing is
scripted in JS as `AppPut`/`AppGet` CAS: opcodes 6 and 7 are **not** sent.

## Exact compiled-artifact pins (SHA-256 verified from bytes at startup)

| role | SHA-256 | source |
|---|---|---|
| kernel `skrynia-server` | `8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f` | secure PR122, source `c39bcbf1` |
| dumb P0 WASM | `d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa` | clean P0 PR53, source `fdf0d027` |
| ordinary P1 WASM (default) | `d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2` | old aa6333a cb-guest export-table pin used by `run.mjs` |
| ordinary P1 WASM (alternate) | `83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a` | PR55 acceptance pin, aa6ffd56 lineage |
| private echo guest | `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5` | disposable register/invoke code |
| trusted edge gateway | `9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f` | source `71773b52` |

P0 frame contract pin: `8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217`.
Only the two exact P1 hashes above are allowlisted; any other substituted byte
string fails the SHA-256 gate. Paths may be overridden with the same
`SKRYNIA_*` variables as `run.mjs`, but hashes are always enforced.

## Run

```sh
cd <repo-root>
# PR58 default old pinned P1
node tests/web-admin-v3/real_native_e2e/p1_admin_ops.mjs

# PR55 acceptance-pinned P1 (same kernel/P0/gateway)
SKRYNIA_P1_WASM=/workspace/skrynia-v3-integration-kit/artifacts/skrynia_native_service_p1clean_83a65a2f.wasm \
  node tests/web-admin-v3/real_native_e2e/p1_admin_ops.mjs

# machine-readable evidence (no secrets)
E2E_EVIDENCE_OUT=/path/evidence.json node tests/web-admin-v3/real_native_e2e/p1_admin_ops.mjs
```

Requires Node with ESM top-level await (verified `v24.21.0`). Disposable
loopback only; server processes are always terminated; temp dirs are removed on
PASS and preserved only for diagnosis on FAIL. Fresh random 32-byte bearer,
operator/admin capabilities and stage bytes per run; bearer stored 0600; no
token, verifier, capability or edge secret is logged or written to EVIDENCE.
Every response is scanned for those values.

## What it proves (42 checks + final PASS)

- **Pins/boot (7):** five exact artifact hashes, kernel boot with P0 root + real
  P1, gateway boot with server-only token.
- **Positive native admin semantics:** register A (op 15); `GET_ACTIVE` (12) =
  ver 3 / live 1 / rev 1 / `sha256(code)`; `LIST_VERSIONS` (13) = `[A]` then
  `[A,B]` sorted after seal, plus `limit=1` and `after=first` pagination;
  `STAGE` (9) returns `sha256(codeB)`; `SEAL` (10) succeeds; `ACTIVATE` (11)
  A→B (rev 1→2) then B→A (rev 2→3) = A/B/A with `GET_ACTIVE` confirming each
  durable active record.
- **No-ready-marker negatives:** activation of B before seal → native err 8;
  activation of a random unsealed hash with the *correct* revision → err 8.
- **CAS / ABA negatives:** stale revision 1 → err 10; replay of revision 1 after
  two activations → err 10; future revision → err 10.
- **Malformed admin frames:** 31-byte activation target → err 1; `limit=0` page
  → err 1; 31-byte admin capability on stage → err 9.
- **Fail-closed credentials/origin:** wrong/missing edge capability → 401;
  wrong origin/host → 403; wrong version (0x00/0x02/0xff), single-bit-wrong
  token, zero token, digest-as-token, and truncated frames (0/1/31/32 bytes)
  → HTTP 500 at the real P0 kernel boundary, no secret in responses.
- **Liveness:** after all denials, `GET_ACTIVE` still returns A at revision 3.

Exact runs on marceline-dev, Node `v24.21.0`, HEAD
`67212c89346e183ac57ca6c0c9c503470ecdc1bb`: both the old pin (`d3686527…`)
and the acceptance pin (`83a65a2f…`) returned **42/42 PASS**; machine-readable
results in `EVIDENCE_ADMIN_OPS_OLDPIN.json` and `EVIDENCE_ADMIN_OPS.json`.

## Precise gap — what this does NOT claim

The shipped PR58 web-admin adapter still implements
`listApps`/`getAppState`/`getReleaseManifest`/`activateRelease` in
`web-admin-v3/native_capabilities.js` using **AppPut/AppGet CAS over appdata
keys** with client-side JSON state, not native opcodes 9/10/11/12/13. The PR58
dev branch also does not contain the `native-service` Rust crate; the admin API
exists in the separately built compiled P1. Therefore:

- This test proves the **real compiled P1 admin ABI is present and correct over
  the same adapter/gateway/kernel/P0 path** — closing the "admin opcodes absent
  from compiled service" blocker.
- It does **not** claim the current UI calls those opcodes. Rewiring
  `native_capabilities.js` to opcodes 9/10/11/12/13 is a separate adapter change
  and is deliberately out of scope here. No false success is claimed for the UI.
- No browser/HTTPS, TLS/DNS, production, deploy, merge, real data or Ivanka
  changes. Kernel and dumb P0 remain generic/dumb and unmodified.
