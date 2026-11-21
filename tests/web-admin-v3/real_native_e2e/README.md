# web-admin-v3 real native HTTP E2E

Disposable, host-local end-to-end qualification of the shipped web-admin-v3
proxy-only Program 0 adapter against the **real pinned compiled stack**. This
is deliberately not a simulator, not a fake/injected capability adapter, and
not a backend-existence claim from front-end tests: it boots the actual kernel,
dumb P0, ordinary WASM service and trusted edge gateway and drives real HTTP.

It qualifies only the transport/authority boundary below; it does **not** claim
the native admin service API (`listApps`/`getAppState`/`getReleaseManifest`/
`activateRelease`) is implemented in WASM, and it does not deploy anything.

```
browser-adapter createBinaryPostTransport()
   | opaque POST bytes + gateway-only session headers
   v
trusted edge gateway  (server-only 32-byte operator token, 0600)
   | prepends 0x01 || raw-token(32) || opaque-body   <-- pinned P0 frame
   v
kernel POST /  ->  dumb P0 (auth + strip 33 bytes)  ->  ordinary nonzero WASM service
```

## Exact compiled-artifact pins (verified by SHA-256 at startup)

| role | SHA-256 | source |
|---|---|---|
| kernel `skrynia-server` | `8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f` | secure PR122 kernel, source `c39bcbf1` |
| dumb P0 WASM | `d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa` | clean P0 PR53, source `fdf0d027` |
| ordinary service WASM | `d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2` | aa6333a cb-guest export-table |
| private echo guest | `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5` | ordinary guest for register/invoke |
| trusted edge gateway | `9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f` | source commit `71773b52` |

P0 wire frame pin referenced by the adapter contract:
`8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217`.
Only version `0x01`, 32-byte SHA256-verified raw token, opaque remainder;
P0 decodes no opcodes and has no root/admin path.

## Run

```sh
cd <repo-root>
node tests/web-admin-v3/real_native_e2e/run.mjs
```

Artifact paths can be overridden with `SKRYNIA_KERNEL_BIN`, `SKRYNIA_P0_WASM`,
`SKRYNIA_P1_WASM`, `SKRYNIA_ECHO_GUEST`, `SKRYNIA_EDGE_GATEWAY_BIN`; the SHA-256
pins are always enforced, so an override cannot silently substitute a binary.
Set `E2E_EVIDENCE_OUT=<path>` to write the machine-readable result.

Verified on host `marceline-dev`, Node `v24.21.0`, at branch HEAD
`3f92c1337de01d9796b6a7d0a4c212a88f9de2a1` (plus this harness). See
`EVIDENCE.json` and the PASS log excerpt below.

## What it proves

- **A (adapter works end-to-end):** a fresh random 32-byte bearer is written
  only to a 0600 gateway token file; the kernel verifier holds its SHA-256. The
  injected fetch adds only gateway session headers (edge capability + origin).
  `createBinaryPostTransport().post()` carries real native payloads through the
  gateway/P0/P1 and back: private guest register, lookup, invoke echo byte-exact,
  AppPut/AppGet round-trip, a bounded CAS update, and a repeatable read of the
  updated value. This demonstrates the proxy-only adapter over the real stack,
  not a mock.
- **B (forged P0 framing fail-closed, real P0):** wrong version `0x00/0x02/0xff`,
  wrong token (single-bit flip), all-zero token, presenting the digest itself as
  the token, and truncated frames (0/1/31/32 bytes) are all denied with HTTP 500
  and no token/verifier material in the response. A subsequent valid read proves
  the kernel survives and state is unchanged.
- **C (gateway auth boundary):** wrong/missing edge capability -> 401; wrong
  origin/host -> 403.

The run also verifies all five artifact SHA-256 pins and re-runs no other suite.
Pre-existing independent suites still pass:
`crypto_vault`, `app_history_activation`, `fail_closed`, `browser_vault_lock`,
`p0_framing_decode` (5/5).

## Scope / not covered

No real browser/HTTPS (host-local HTTP gateway only); no production TLS/DNS;
no CLI, kernel/gateway source changes, deploy, merge or release; no native admin
service API implementation. The service operations above exercise the ordinary
guest's proven native register/invoke/AppPut/AppGet/CAS ABI, not admin semantics.
Per AGENTS.md, no credentials are placed in public assets, URLs, Git history or
logs; all tokens are disposable and generated per run.

## PASS log excerpt (exact run, 2026-11-21T23:00Z)

```
PASS pin kernel sha256 8da114ff4c6c :: secure PR122 kernel, source c39bcbf1
PASS pin p0 sha256 d57ab27aacc0 :: clean dumb P0 PR53, source fdf0d027
PASS pin service sha256 d3686527bbf2 :: ordinary nonzero WASM service, aa6333a cb-guest export table
PASS pin echo sha256 dc28e490b107 :: ordinary private guest used only to exercise service register/invoke
PASS pin gateway sha256 9fa4df141d5b :: trusted edge gateway, source commit 71773b52
PASS pinned kernel booted with P0 root + ordinary P1 service
PASS trusted edge gateway booted with server-only operator token
PASS adapter bound to trusted gateway endpoint
PASS A1 register ordinary private guest via adapter :: native status 0, body 32B
PASS A2 lookup registered guest via adapter :: native status 0, body 170B
PASS A3 invoke ordinary guest via adapter :: native status 0, body 23B
PASS A3b ordinary service echoed payload byte-exact
PASS A4 AppPut initial value via adapter :: native status 0, body 32B
PASS A5 AppGet initial value via adapter :: native status 0, body 9B
PASS A5b value round-tripped through P0 to ordinary service
PASS A6 bounded CAS via adapter :: native status 0, body 32B
PASS A7 AppGet after CAS via adapter :: native status 0, body 9B
PASS A7b CAS updated value through the real stack
PASS A8 repeatable read (rollback target readable)
PASS A8b repeat read stable
PASS B1 wrong version 0x00 denied :: http 500
PASS B2 wrong version 0x02 denied :: http 500
PASS B3 wrong version 0xff denied :: http 500
PASS B4 wrong token denied :: http 500
PASS B5 zero token denied :: http 500
PASS B6 digest-as-token denied :: http 500
PASS B7 empty frame denied :: http 500
PASS B8 1-byte frame denied :: http 500
PASS B9 31-byte truncated header denied :: http 500
PASS B10 32-byte truncated header denied :: http 500
PASS B11 kernel alive after forged frames :: native status 0, body 9B
PASS B11b state unchanged after denial
PASS C1 wrong edge capability 401 :: http 401
PASS C2 missing edge capability 401 :: http 401
PASS C3 wrong origin 403 :: http 403
PASS C4 wrong host 403 :: http 403
PASS web-admin-v3 real native HTTP E2E: adapter+trusted gateway+kernel+P0+ordinary service end-to-end, forged P0 framing fail-closed
```
