# Disposable compiled publication + Chromium bootstrap fixture

Run `node bootstrap/tests/publication_bootstrap_e2e.mjs` from the tools
checkout on marceline-dev. It reuses the already compiled exact kit at
`/workspace/skrynia-exact-kit-73d8fa1` (release kernel, publication root,
publish provider); it does not rebuild, deploy, modify production config,
DNS, TLS or credentials. All roots and ports are disposable and loopback-only.

The fixture boots the compiled reference provider and a kernel with the
compiled publication root WASM, runs the shipped `native-publish.mjs` through
the one-time installer, independently rehashes every served file, then drives
the published unlock page in real Chromium (wrong password, correct password,
scoped native session, reload re-lock). Logs and SHA-256 digests are printed
for the board record.
