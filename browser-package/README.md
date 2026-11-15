# Skrynia Tools browser client

The browser package provides `wasm-bindgen` generated JavaScript/TypeScript and WebAssembly bindings for Skrynia Tools. It uses browser `window.fetch`, so it is not a Node.js server HTTP client.

```js
import init, { BrowserClient, sha256Hex } from "@ottojung/skrynia-tools-browser";
await init();
const skrynia = new BrowserClient("https://skrynia.example", null);
const value = await skrynia.get("public.document");
```

To write, pass a **narrowly scoped user token** as the second argument. Never bundle issuer/root credentials. Follow browser CORS, cache and cross-origin security rules.

This package is assembled from the `skrynia-tools` Rust crate's release Wasm build and matching `wasm-bindgen` generator in CI. The repository is AGPL-3.0-or-later. Publishing to a public npm registry is a separate action requiring repository owner approval.

The release tarball includes `LICENSE` and both the generated JS/TS bindings and `.wasm` bytes. Run `npm pack` inside the assembled output to create an installable `.tgz` artifact.
