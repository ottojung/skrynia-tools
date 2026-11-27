# Disposable compiled publication + Chromium bootstrap fixture

The Native components workflow's `bootstrap-real-e2e` job downloads the
compiled-stack and publication artifacts, verifies source revisions and
SHA-256 manifests, and runs this fixture against those exact binaries.

For a local replay, provide:

- `SKRYNIA_KIT`: directory containing `kernel-release`, `publication-root.wasm`
  and `provider`, with their expected SHA-256 hashes in
  `SKRYNIA_KIT_KERNEL_SHA256`, `SKRYNIA_KIT_PUBLICATION_ROOT_SHA256` and
  `SKRYNIA_KIT_PROVIDER_SHA256`.
- The five ordinary-stack artifact paths and expected hashes documented in
  the workflow (kernel, P0, P1, echo guest and trusted edge).
- `NGINX_BIN`, `CHROMIUM_BIN` and `SKRYNIA_PLAYWRIGHT_MODULE`: explicit paths
  to the local nginx executable, Chromium executable and Playwright module.

Then run `node bootstrap/tests/publication_bootstrap_e2e.mjs` from the
tools checkout. Missing inputs or mismatched artifact digests fail closed.
No default historical kit is used. The fixture does not rebuild, deploy,
modify production config, DNS, TLS or credentials. All roots and ports are
disposable and loopback-only.

The fixture boots the compiled reference provider and a kernel with the
compiled publication root WASM, runs the shipped `native-publish.mjs` through
the one-time installer, independently rehashes every served file, then drives
the published unlock page in real Chromium (wrong password, correct password,
scoped native session, reload re-lock). Logs and SHA-256 digests are printed
for the board record.
