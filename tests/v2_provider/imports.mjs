import { readFileSync } from 'node:fs';
import { strict as assert } from 'node:assert';

const path = process.argv[2];
assert.ok(path, 'Supply the compiled unprivileged v2 guest Wasm file');
const module = new WebAssembly.Module(readFileSync(path));
const imports = WebAssembly.Module.imports(module);
assert.deepEqual(imports, [
  { module: 'skrynia', name: 'invoke', kind: 'function' },
]);
const exports = WebAssembly.Module.exports(module).map(x => x.name);
assert.ok(exports.includes('memory'), 'Guest must export its linear memory');
assert.ok(exports.includes('skrynia_main'), 'Guest must export the child entry');
console.log('Native v2 guest imports only skrynia.invoke: PASS');
