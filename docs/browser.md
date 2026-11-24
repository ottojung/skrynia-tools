# Browser / WebAssembly SDK

The browser and Rust clients share the ordinary native service wire format,
content digest and append-log encoding. The browser uses asynchronous fetch.

## Build

Use a Rust WASM target and matching wasm-bindgen CLI.

```sh
cargo build --locked --release --target wasm32-unknown-unknown --lib --no-default-features
wasm-bindgen --target web --out-dir dist/skrynia-wasm \
  target/wasm32-unknown-unknown/release/skrynia_tools.wasm
bash tests/browser/package.sh
```

The resulting local package is not a claim of npm publication.

```js
import init, { BrowserClient, sha256Hex } from "@ottojung/skrynia-tools-browser";
await init();
const client = new BrowserClient(
  "https://app.example/_skrynia/", edgeCapability, readCapability, writeCapability
);
const bytes = new TextEncoder().encode("hello");
await client.create("sample.hello", bytes);
const current = await client.get("sample.hello");
if (current !== undefined) console.log(new TextDecoder().decode(current));
```

The edge capability is supplied separately from the 32-byte hexadecimal read
capability and optional corresponding write capability. The read-only form
omits the fourth argument. No root or operator credential belongs in browser
JavaScript.

## API

- `get(key)`: optional byte array.
- `create(key, bytes)`: true on confirmed creation, false on conflict.
- `replace(key, bytes, expectedSha256)` and `delete(key, expectedSha256)`:
  confirmed success, explicit conflict, or exception.
- `initializeLog(prefix)`: create or verify genesis.
- `appendLog(prefix, operationId, bytes)`: returns a BigInt sequence.
- `readLog(prefix, start, limit)`: bounded contiguous read as JSON.
- `sha256Hex(bytes)`: SHA-256 of raw bytes.

There is no enumeration interface in the native data client. A caller needing
unordered discovery must supply an independent event index. All data operations
use binary POST. The client does not bypass origin policy. Uncertain mutation
responses must be reconciled against authoritative state.

## Testing

The browser fixture exercises frame validation, scoped read/write capabilities,
conditional operations, concurrent append and unexpected transport responses.
CI builds a single WASM artifact and tests Chromium, Firefox and WebKit against
isolated fixtures. These tests are not production or compiled-stack acceptance.

```sh
bash tests/browser/run.sh
npm ci --prefix tests/browser
./tests/browser/node_modules/.bin/playwright install --with-deps firefox webkit
node tests/browser/cross_browser.mjs
```
