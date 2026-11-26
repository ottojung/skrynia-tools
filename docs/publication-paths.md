# Publication path inventory

Target integration path inventory; not a release approval. The explicit
`operator/` and `user/` namespaces are the K10 migration convention, pending
whole-stack qualification. Prefixes select existing private roots and do not
change paths relative to those roots or require relocating persisted data.

| Path / surface | Root | Kind | Publication status |
|---|---|---|---|
| `operator/security/program0.sha256` | private operator disk | token verifier | never published; read by Program 0 and the publication root to verify the token |
| `operator/programs/root.wasm` | private operator disk | normal root program | private; Program 0 loads it with single-function `eval` |
| `user/programs/operator.cap` | private user disk | test operator capability | private |
| `user/publication-requests/<8 hex>` | private root-writable user disk | request-only mailbox | create-only u32 ticket, bounded UTF-8 relative path, read-only to provider, removed by the root after the import |
| `user/programs/active/<id>.bin` | private user disk | mutable code pointer | private; revoke any public entry before replacement |
| `user/programs/blob/<hash>.bin` | private user disk | immutable sealed code | private |
| `user/programs/ready/<id>/<hash>.ready` | private user disk | ready marker | private |
| `user/programs/appdata/<cap>/<hash>.bin` | private user disk | application data | private, never published |
| `apps/<app>/...` | separate nginx root | releases, stable assets, active pointer | only publishable namespace; explicit syscall, never a storage alias |

## Actual WASM publication lifecycle

`skrynia-publication-root` is a separate root WASM built from this repository.
It imports `syscall.publish`/`syscall.unpublish` directly, authenticates the
operator token, validates generic path grammar and private-root exclusion,
writes the mailbox, invokes the import synchronously and returns the outcome.
Ordinary Program 0 does **not** import either publication syscall; children get no
publication authority. The outcome contract is in
[publication-root/README.md](../publication-root/README.md).

The merged `skrynia-apps/scripts/publish-static-v3.mjs` still POSTs directly to
the provider. That is an operator convenience path, **not** an actual WASM
lifecycle and not evidence that a guest published anything. The real lifecycle
is exercised by `publication-root/tests/real_wasm_chain.py` with the compiled
kernel, the produced root WASM, the real provider and real nginx.

## Configuration and deploy integration

- Discovery is `GET <base>/syscalls_list` with ordered typed function metadata.
  Both operations accept one i32 and return one i32, encoded as the seven-byte
  version-1 frame `[1, 1, 0x32, u32-le]`. Startup fails closed if either
  imported name or signature is unavailable.
- The mailbox remains the private user root's `publication-requests/` directory;
  the target ABI addresses it as `user/publication-requests/`. The provider mounts it read-only;
  nginx never mounts it and children cannot scope into it.
- Provider roots are disjoint: `PATH_TO_SKRYNIA_DISK` holds source bytes,
  `PATH_TO_NGINX_SKRYNIA_ROOT` holds only publication symlinks. The full
  relative path is mirrored, e.g. `apps/x/releases/<id>/a.js` becomes
  `<nginx-root>/apps/x/releases/<id>/a.js`.
- skrynia-apps nginx configs keep `disable_symlinks on` globally and enable it
  only in publication locations. No private storage is directly servable.
- A stable public entry (for example `active.json`) is published once as a
  symlink to an immutable source target. A-B-A version switching and rollback
  rewrite only that private pointer atomically: the public symlink is retained,
  both immutable releases remain available and no unpublish/republish gap is
  introduced. A named published asset is revoked before its backing bytes are
  deleted or repurposed; an immutable-pointer switch is not such a revocation.
  The CAS A-B-A switch is covered by the real-chain fixture.
