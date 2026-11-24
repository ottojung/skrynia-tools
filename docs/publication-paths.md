# Publication path inventory

Observed v3 paths at the canonical PR98 integration head plus the merged
provider and apps publication helpers. This is an inventory, not a release
approval.

| Path / surface | Root | Kind | Publication status |
|---|---|---|---|
| `security/program0.sha256` | private operator disk | token verifier | never published; read by Program 0 and the publication root after token check |
| `programs/root.wasm` | private operator disk | normal root program | private; Program 0 loads it with `eval_delegate` |
| `programs/operator.cap` | private operator disk | test operator capability | private |
| `publication-requests/<8 hex>` | private root-writable user disk | request-only mailbox | create-only u32 ticket, bounded UTF-8 relative path, read-only to provider, removed by the root after the import |
| `programs/active/<id>.bin` | private user disk | mutable code pointer | private; revoke any public entry before replacement |
| `programs/blob/<hash>.bin` | private user disk | immutable sealed code | private |
| `programs/ready/<id>/<hash>.ready` | private user disk | ready marker | private |
| `programs/appdata/<cap>/<hash>.bin` | private user disk | application data | private, never published |
| `apps/<app>/...` | separate nginx root | releases, stable assets, active pointer | only publishable namespace; explicit syscall, never a storage alias |

## Actual WASM publication lifecycle

`skrynia-publication-root` is a separate root WASM built from this repository.
It imports `syscall.publish`/`syscall.unpublish` directly, authenticates the
operator token, validates generic path grammar and private-root exclusion,
writes the mailbox, invokes the import synchronously and returns the outcome.
Program 0 is unchanged and does **not** import any syscall; children get no
publication authority. The outcome contract is in
[publication-root/README.md](../publication-root/README.md).

The merged `skrynia-apps/scripts/publish-static-v3.mjs` still POSTs directly to
the provider. That is an operator convenience path, **not** an actual WASM
lifecycle and not evidence that a guest published anything. The real lifecycle
is exercised by `publication-root/tests/real_wasm_chain.py` with the compiled
kernel, the produced root WASM, the real provider and real nginx.

## Configuration and deploy integration

- Discovery is `GET <base>/syscalls_list` (HTTP 200, plain JSON array of
  strings), superseding `POST /version` provider discovery. Per-call
  `POST <base>/<name>` still uses exactly four little-endian bytes. Startup
  fails closed if either imported name is unavailable.
- The kernel's write default is the private user root, so the mailbox is that
  root's `publication-requests/` directory. The provider mounts it read-only;
  nginx never mounts it and children cannot scope into it.
- Provider roots are disjoint: `PATH_TO_SKRYNIA_DISK` holds source bytes,
  `PATH_TO_NGINX_SKRYNIA_ROOT` holds only publication symlinks. The full
  relative path is mirrored, e.g. `apps/x/releases/<id>/a.js` becomes
  `<nginx-root>/apps/x/releases/<id>/a.js`.
- skrynia-apps nginx configs keep `disable_symlinks on` globally and enable it
  only in publication locations. No private storage is directly servable.
- Version switching and rollback revoke the old public path before replacing
  the mutable private pointer, then publish the selected immutable target.
  A CAS A-B-A switch is covered by the real-chain fixture.
