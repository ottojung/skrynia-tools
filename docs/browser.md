# Browser / WebAssembly SDK

Skrynia Tools has a browser-oriented Rust module built with `wasm-bindgen`. The browser and native SDK share the same Rust byte-level log entry format, key layout, operation-ID validation and duplicate-detection logic. Their networking transports are different: the native client uses HTTP and the browser adapter uses `window.fetch`.

## Build

Install the Rust `wasm32-unknown-unknown` standard library and a `wasm-bindgen-cli` version matching the crate's resolved `wasm-bindgen` version.

```sh
cargo build --locked --release --target wasm32-unknown-unknown --lib --no-default-features
wasm-bindgen --target web --out-dir dist/skrynia-wasm \
  target/wasm32-unknown-unknown/release/skrynia_tools.wasm
```

Then import the generated module, call its initialization function, and construct `BrowserClient`. Example using the npm package (or replace the import with a relative file URL):

```js
import init, { BrowserClient, sha256Hex } from "@ottojung/skrynia-tools-browser";
await init();
const client = new BrowserClient("https://skrynia.example", publishToken);
const bytes = new TextEncoder().encode("hello");
await client.create("sample.hello", bytes);
const current = await client.get("sample.hello");
console.log(new TextDecoder().decode(current));
```

The user token is optional for public object reads; provide a **narrowly scoped user token** for authenticated writes, listing and log appends. Browser code and its credentials are accessible to its users. Do not embed root or issuer tokens.

## API

- `BrowserClient(url, userToken?)`: validated server base URL with optional bearer user token
- `get(key)`: optional `Uint8Array`
- `create(key, bytes)`: `true` on create, `false` on 409
- `replace(key, bytes, expectedSha256?)`, `delete(key, expectedSha256?)`: `false` on 412, otherwise success or exception
- `listPage(locations, after?)`: JSON string for one bounded server page
- `initializeLog(prefix)`: create or verify genesis
- `appendLog(prefix, operationId, bytes)`: returns a `BigInt` sequence number
- `readLog(prefix, start, limit)`: JSON string with hex payloads, capped at 1000 entries
- `sha256Hex(bytes)`: lowercase SHA-256 of raw bytes for content-hash CAS

The browser adapter is asynchronous. CI now assembles an installable `@ottojung/skrynia-tools-browser` npm tarball with the generated `wasm-bindgen` JavaScript, TypeScript declarations, WebAssembly binary, source license and README. The package is **not published to the npm registry**: download the CI artifact and install the `.tgz` file, or build it locally with `bash tests/browser/package.sh` after generating bindings.

The `wasm-bindgen`-generated JS API remains the typed interface; a hand-written TypeScript façade is not yet provided. The package is browser-oriented, not a Node HTTP client.

## Security and operational constraints

Network access requires browser-origin CORS permissions to the Skrynia server. The SDK does not bypass same-origin restrictions. Skrynia object reads are public. Mutable object reads and writes need authoritative uncached content where CAS is involved. A Skrynia log assumes cooperating create-only writers; malicious writers can block progress. Lost HTTP replies may leave mutations committed, so clients must reconcile durable state before retrying. This initial Wasm binding has not yet undergone real-browser security and interoperability tests across Chrome/Firefox/WebKit.
