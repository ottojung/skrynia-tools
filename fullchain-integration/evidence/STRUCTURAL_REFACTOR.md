# Fullchain structural refactor evidence (DRAFT, marceline-dev)

Owner-sequenced bounded refactor after `[pr79-evidence-fix]` finished. Base:
PR #80 head `ecdc67ebd69e237549319b1924b5669e35762c7f` (stacked on PR #79
`30bb38ca`). No merge, no deploy, no real VAPID credential/push service, no
Chromium/five-app migration, no DNS/TLS/production data, no cache deletion, no
kernel/WASM/Podman rebuild.

## LOC before -> after (owner method: `awk 'NF && $0 !~ /^[[:space:]]*\/\//'`)

| file | before | after |
| --- | ---: | ---: |
| `scenarios/negative/p1_caps.rs` | 75 | 14 |
| `scenarios/negative/push.rs` | 92 | 20 |
| `chain/edge.rs` | 84 | 44 |
| `chain/kernel.rs` | 78 | 50 |
| `provider/tls/cert.rs` | 100 | 29 |
| `provider/server/run/accept.rs` | 93 | 45 |
| `provider/server/run/types.rs` | 81 | 36 |
| `provider/server/http.rs` | 61 | 2 |
| `provider/decrypt.rs` | 61 | 37 |
| `provider/vapid.rs` | 54 | 36 |
| `chain/pins.rs` (part's new file) | 61 | 41 |

The 10 owner-listed counts differ slightly from the original 01:08Z review
because that review measured PR #79 `30bb38ca`; the numbers above are measured
on PR #80 `ecdc67e`, which adds `pins.rs` and the wrong-version negative.

All split modules are cohesive (`edge_cfg`, `kernel_fs`, `proc_ready`,
`pins/table`, `cert_gen`/`cert_ca`/`cert_leaf`, `decrypt_derive`, `vapid_parse`,
`http_req`/`http_reply`, `accept_handle`, `fake_push`, `p1_caps_support`/
`p1_caps_checks`/`p1_caps_report`, `push_setup`/`push_pump`/`push_one`).
Max directory entries after split: `scenarios/negative` = 10, `chain` = 10,
all others <= 8. No file exceeds 50 code LOC by the owner `awk` method or by
`source_lints::code_lines`.

All cross-module imports were changed to flat root re-exports (or one-hop
`super::sibling`), so the sibling-only import lint also passes over the harness.

## CI wiring

- `source_lints/mod.rs`: `check()` now visits `fullchain-integration/src`
  (plus tests/examples/benches when present).
- `build.rs`: `check(&root.join("fullchain-integration"))` plus matching
  `rerun-if-changed`, so the existing root `rust` job's
  `cargo test --locked --all-targets` **already enforces** the harness's 50-LOC,
  <=10-entry and sibling-import rules in CI.
- `.github/workflows/ci.yml` **not applied / CREDENTIAL-BLOCKED**: the only
  available GitHub credential is a `gho_` OAuth token whose scopes are
  `admin:public_key, gist, read:org, repo` — no `workflow` scope. `git push`
  rejects any ref that updates `.github/workflows/ci.yml` with:
  `refusing to allow an OAuth App to create or update workflow ... without workflow scope`.
  The exact ready-to-apply job (verified locally: fmt rc=0, test 4/4, clippy
  rc=0) is included verbatim in the DRAFT PR body. An operator with a
  workflow-scoped credential must add that job to `ci.yml`.

## Negative mutation nonvacuity

Gate driver (compile-free, mirrors build.rs checks):

```
rustc --edition 2021 -o /tmp/pr80-lint-driver /tmp/pr80-lint-driver.rs
/tmp/pr80-lint-driver    # source_lints::check(root)
```

- RED: added `fullchain-integration/src/red_probe.rs` with 60 code LOC ->
  `Rust structural lint: .../red_probe.rs: 60 code LOC exceeds 50`, panic
  `1 Rust structural lint violations`, exit 101.
- GREEN: removed the probe -> exit 0 (`ALL_STRUCTURAL_GATES_OK`).
- RED (fmt): appended `fn  broken_format( )->u8{1}` to
  `fullchain-integration/src/evidence/mod.rs` ->
  `cargo fmt --manifest-path fullchain-integration/Cargo.toml --all -- --check`
  reported a diff and exit 1.
- GREEN: restored the file -> exit 0.

## Preserved part artifacts

Refactored harness stdout is byte-identical (sha256
`20816423fd2aa4681405d499092555abc420a3b09e5469416a421769c363505c`) to the
captured stdout in `evidence/FULLCHAIN_OK_2bc4a20.log`. All five pinned
artifacts still pass `artifacts/MANIFEST.sha256.txt`; runtime SHA pin enforcement
in `chain/pins.rs` and its 4 RED/GREEN tests are unchanged in behavior.
