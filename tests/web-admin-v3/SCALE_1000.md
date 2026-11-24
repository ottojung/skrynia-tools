# Pinned 1000-version / restart acceptance gate (real compiled stack)

`p1_admin_scale_1000.mjs` is a permanent, non-vacuous test of the
1000-historical-version and restart-durability acceptance items for the
authenticated ordinary-P1 code registry. It is a test only — no CLI, no
simulator, no `AppPut`/`AppGet` stand-in, no rebuild, no deploy, no real data.

```
shipped binary POST adapter -> pinned trusted edge gateway -> pinned release kernel
  -> pinned dumb P0 (SHA256 token gate, 33-byte strip, decodes no opcodes)
  -> pinned ordinary compiled P1 (opcodes 9/10/11/12/13/14 in WASM)
```

Exact pins are hardened in the test source and re-verified from bytes at
startup: release kernel `a1dc4119…` (`kernel-release`, source `806c172…`),
P0 `44e845a4…`, P1 `c7239fc5…`, echo guest `dc28e490…`, gateway
`9fa4df14…`. Paths default to `/workspace/skrynia-exact-kit-73d8fa1` and may
be redirected with `SKRYNIA_KIT_DIR`, but substituted bytes cannot pass the
hash gate.

## Run

```sh
node tests/web-admin-v3/p1_admin_scale_1000.mjs
E2E_EVIDENCE_OUT=/path/evidence.json node tests/web-admin-v3/p1_admin_scale_1000.mjs
```

Requires Node with ESM top-level await (verified `v24.21.0`).

## What it proves at 1000-version scale

- 1000 distinct `STAGE`+`SEAL` pairs over the real opcodes succeed; the active
  pointer is untouched by 2000 staging/sealing operations.
- O(1) activation: with the full history resident, `ACTIVATE` is a fixed 76-byte
  request and 42-byte active record. A full private-state fingerprint before
  and after each activation shows exactly one changed file — the fixed 42-byte
  active record — with every code blob and sealed-ready marker byte-identical.
  No files are added or removed (no migration, no history scan in the write
  path).
- Monotone `u64` revision across arbitrary earlier/later activations
  (`rev1→2→3→4→5`), A-B-A across history, and stale/ABA/future/unsealed/
  staged-but-unsealed/forged-capability/corrupt-marker/corrupt-blob/malformed
  frame denials, with liveness intact.
- Authority separation: trusted operator install cap, private admin cap and
  public invoke/read ID are distinct; a foreign cap has no authority over the
  object. Forged P0 frames and wrong gateway credentials fail closed without
  leaking secrets.
- Kill/restart of kernel+gateway on the same private dirs keeps every private
  file byte-identical, resolves the exact committed active record, keeps
  uncommitted staging unactivatable, and continues with the next revision.
- A bounded 61-version control object proves the shipped adapter's cursor
  pagination, drop-after-commit `uncertain` handling, no blind CAS replay,
  refresh reconciliation and post-restart continuation.

## Known platform ceiling (honest FAIL, not a false pass)

`LIST_VERSIONS` over the 1000-version object returns native err 7 (`IO`) on
this exact pinned stack. Independent diagnosis shows the boundary at **198
ready markers OK / 199 FAIL**, for page limits 1, 16 and 256 alike: the native
`list_ready` enumerates and reads every marker before slicing the page, so
pagination cannot avoid the ceiling. The three gate checks that depend on
listing 1001 versions (`SCALE-LIST`, `ADAPTER-scale`, `RST5`) therefore FAIL by
design until the kernel/gateway callback budget is raised or the native list
path is made incremental. Everything else at 1000-version scale passes.

This is DEV acceptance evidence only: not a release, merge, cutover, TLS/DNS
or production approval.
