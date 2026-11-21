#!/usr/bin/env node
// Deterministic in-memory proof of the semantic adapter's wire translation and
// O(1) CAS activation. This is a contract test only: the real compiled stack
// qualification is tests/web-admin-v3/real_native_e2e/chromium.mjs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve, dirname} from 'node:path';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const {createNativeCapabilities, serializeState} = await import(
  new URL('../../web-admin-v3/native_capabilities.js', import.meta.url).href);

const sha = b => createHash('sha256').update(Buffer.from(b)).digest();
const putId = b => { const x = Buffer.from(b); const h = Buffer.alloc(2); h.writeUInt16LE(x.length); return Buffer.concat([h, x]); };
const putBlob = b => { const x = Buffer.from(b); const h = Buffer.alloc(4); h.writeUInt32LE(x.length); return Buffer.concat([h, x]); };
const appPutOp = (cap, key, val) => Buffer.concat([Buffer.from([1, 6]), putId(cap), putBlob(key), Buffer.from([0]), putBlob(val)]);
const envelope = (status, body = Buffer.alloc(0)) => {
  const h = Buffer.alloc(6); h[0] = 1; h[1] = status; h.writeUInt32LE(body.length, 2);
  return Buffer.concat([h, body]);
};
function readBlob(b, at) { const len = b.readUInt32LE(at); return {data: b.subarray(at + 4, at + 4 + len), next: at + 4 + len}; }
function readId(b, at) { const len = b.readUInt16LE(at); return {data: b.subarray(at + 2, at + 2 + len), next: at + 2 + len}; }

function fakeService() {
  const store = new Map();
  const adminCap = sha('admin');
  const readCap = sha(adminCap);
  function put(key, value) { store.set(key, Buffer.from(value)); }
  function handle(bytes) {
    const b = Buffer.from(bytes);
    assert.equal(b[0], 1, 'version byte');
    if (b[1] === 6) {
      let at = 2;
      const idCap = readId(b, at); at = idCap.next;
      assert.deepEqual(idCap.data, adminCap, 'admin capability');
      const key = readBlob(b, at); at = key.next;
      const mode = b[at]; at += 1;
      if (mode === 0) {
        const val = readBlob(b, at);
        put(key.data.toString(), val.data);
        return envelope(0);
      }
      const expected = b.subarray(at, at + 32); at += 32;
      const val = readBlob(b, at);
      const current = store.get(key.data.toString());
      if (!current || !sha(current).equals(expected)) return envelope(1);
      put(key.data.toString(), val.data);
      return envelope(0);
    }
    if (b[1] === 7) {
      let at = 2;
      const idCap = readId(b, at); at = idCap.next;
      assert.deepEqual(idCap.data, readCap, 'read capability');
      const key = readBlob(b, at);
      const found = store.get(key.data.toString());
      return found ? envelope(0, found) : envelope(1);
    }
    throw Error('unexpected opcode ' + b[1]);
  }
  const fetchImpl = async (_url, init) => {
    const body = handle(Buffer.from(init.body));
    return {status: 200, arrayBuffer: async () => body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength)};
  };
  return {store, adminCap, readCap, fetchImpl, put};
}

function manifest(app, id, commit) {
  return {schema: 'skrynia-native-static-v1', app, release: id, commit, repo: 'r', files: [{path: 'index.html', sha256: id}]};
}

let ok = 0;
const check = (name, cond, detail = '') => {
  assert.ok(cond, name + (detail ? ': ' + detail : ''));
  ok += 1;
  console.log('PASS ' + name + (detail ? ' :: ' + detail : ''));
};

const svc = fakeService();
const A = 'a'.repeat(64), B = 'b'.repeat(64);
const app = 'demo';
svc.put('v3admin:apps', JSON.stringify([app]));
svc.put('v3admin:app:demo:state', serializeState({active: A, pointerVersion: 4}));
svc.put('v3admin:app:demo:versions', JSON.stringify([
  {id: A, ready: true, manifest: manifest(app, A, 'ca')},
  {id: B, ready: true, manifest: manifest(app, B, 'cb')},
  {id: 'c'.repeat(64), ready: false, manifest: null},
]));
const caps = createNativeCapabilities({
  endpoint: 'http://127.0.0.1:1/_skrynia/',
  readCap: svc.readCap.toString('hex'),
  adminCap: svc.adminCap.toString('hex'),
}, svc.fetchImpl);

assert.deepEqual(await caps.listApps(), [app]);
check('listApps decodes native AppGet', true);
const state = await caps.getAppState(app);
check('getAppState lists all historical versions', state.versions.length === 3);
check('getAppState marks unsealed version unusable', state.versions.filter(v => !v.usable).length === 1);
check('getAppState reports A/B active pointer + revision', state.active === A && state.pointerVersion === 4);
const m = await caps.getReleaseManifest(app, B);
check('getReleaseManifest returns normalized historical manifest', m.commit === 'cb' && m.files === 1);

const switched = await caps.activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 4});
check('CAS activation to later release is one pointer write', switched.active === B && switched.pointerVersion === 5);
const back = await caps.activateRelease(app, A, {expectedActive: B, expectedPointerVersion: 5});
check('A-B-A rollback to earlier release advances revision', back.active === A && back.pointerVersion === 6);
await assert.rejects(
  caps.activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 4}),
  e => e.code === 'conflict');
check('stale expected revision is rejected as conflict (no replay)', true);
const after = await caps.getAppState(app);
check('conflict left the durable pointer unchanged', after.active === A && after.pointerVersion === 6);
await assert.rejects(
  caps.activateRelease(app, 'c'.repeat(64), {expectedActive: A, expectedPointerVersion: 6}),
  e => e.code === 'invalid_state');
check('activation of unsealed release fails closed locally', true);

console.log(`PASS native capability adapter contract (${ok} checks)`);
