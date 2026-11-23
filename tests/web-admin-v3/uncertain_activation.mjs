#!/usr/bin/env node
// Regression contract for uncertain activation outcomes.
//
// A lost ACK, a gateway 5xx or a malformed 2xx response after a single CAS
// pointer write leaves the outcome unknown. AGENTS.md requires that such an
// outcome be distinguished from a definitive conflict/denial and never be
// silently replayed. This test is deterministic (injected fetch + DOM), does
// not prove a live backend, and runs without any Rust/WASM build.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';

const sha = b => createHash('sha256').update(Buffer.from(b)).digest();
const putId = b => {
  const x = Buffer.from(b), h = Buffer.alloc(2);
  h.writeUInt16LE(x.length);
  return Buffer.concat([h, x]);
};
const putBlob = b => {
  const x = Buffer.from(b), h = Buffer.alloc(4);
  h.writeUInt32LE(x.length);
  return Buffer.concat([h, x]);
};
const envelope = (status, body = Buffer.alloc(0)) => {
  const h = Buffer.alloc(6);
  h[0] = 1; h[1] = status; h.writeUInt32LE(body.length, 2);
  return Buffer.concat([h, body]);
};
function readBlob(b, at) {
  const len = b.readUInt32LE(at);
  return {data: b.subarray(at + 4, at + 4 + len), next: at + 4 + len};
}
function readId(b, at) {
  const len = b.readUInt16LE(at);
  return {data: b.subarray(at + 2, at + 2 + len), next: at + 2 + len};
}

const {createNativeCapabilities, serializeState} = await import(
  new URL('../../web-admin-v3/native_capabilities.js', import.meta.url).href);
const {createAppsController} = await import(
  new URL('../../web-admin-v3/apps_ui.js', import.meta.url).href);

const A = 'a'.repeat(64), B = 'b'.repeat(64);
const app = 'demo';
const manifest = id => ({
  schema: 'skrynia-native-static-v1', app, release: id, commit: 'c', repo: 'r',
  files: [{path: 'index.html', sha256: id}],
});

// A fake service whose CAS behaviour is injected. Counts every CAS POST so the
// "no silent replay" obligation is directly observable.
function fakeService(casMode) {
  const store = new Map();
  const adminCap = sha('admin');
  const readCap = sha(adminCap);
  let casAttempts = 0;
  store.set('v3admin:apps', Buffer.from(JSON.stringify([app])));
  store.set('v3admin:app:demo:state',
    Buffer.from(serializeState({active: A, pointerVersion: 4})));
  store.set('v3admin:app:demo:versions', Buffer.from(JSON.stringify([
    {id: A, ready: true, manifest: manifest(A)},
    {id: B, ready: true, manifest: manifest(B)},
  ])));

  function cas(bytes) {
    casAttempts += 1;
    let at = 2;
    at = readId(bytes, at).next;
    at = readBlob(bytes, at).next;
    at += 1; // mode byte
    const expected = bytes.subarray(at, at + 32);
    const value = readBlob(bytes, at + 32).data;
    const current = store.get('v3admin:app:demo:state');
    const matched = current && sha(current).equals(expected);
    if (matched) store.set('v3admin:app:demo:state', Buffer.from(value));
    if (casMode === 'conflict') return envelope(1); // definitive service rejection
    if (casMode === 'malformed') return Buffer.from([1, 0, 9, 9, 9, 9]); // bad length
    if (casMode === 'server_error') {
      return {httpStatus: 503, body: envelope(1)};
    }
    if (casMode === 'lost_ack') return {networkError: true};
    return envelope(matched ? 0 : 1);
  }

  function handle(bytes) {
    const b = Buffer.from(bytes);
    assert.equal(b[0], 1, 'version byte');
    if (b[1] === 6) return cas(b);
    if (b[1] === 7) {
      let at = 2;
      const idCap = readId(b, at); at = idCap.next;
      assert.deepEqual(idCap.data, readCap);
      const key = readBlob(b, at);
      const found = store.get(key.data.toString());
      return found ? envelope(0, found) : envelope(1);
    }
    throw Error('unexpected opcode ' + b[1]);
  }

  const fetchImpl = async (_url, init) => {
    const out = handle(Buffer.from(init.body));
    if (out && out.networkError) {
      const e = Error('socket hang up');
      e.code = 'ECONNRESET';
      throw e;
    }
    if (out && out.httpStatus) {
      return {status: out.httpStatus, arrayBuffer: async () => out.body};
    }
    return {status: 200, arrayBuffer: async () => out.buffer.slice(out.byteOffset, out.byteOffset + out.byteLength)};
  };
  return {store, adminCap, readCap, fetchImpl, get casAttempts() { return casAttempts; }};
}

function capabilitiesFor(svc) {
  return createNativeCapabilities({
    endpoint: 'http://127.0.0.1:1/_skrynia/',
    readCap: svc.readCap.toString('hex'),
    adminCap: svc.adminCap.toString('hex'),
  }, svc.fetchImpl);
}

let ok = 0;
const check = (name, cond, detail = '') => {
  assert.ok(cond, name + (detail ? ': ' + detail : ''));
  ok += 1;
  console.log('PASS ' + name + (detail ? ' :: ' + detail : ''));
};

// ---------- A) adapter: distinguish unknown from definitive outcomes ----------
for (const mode of ['lost_ack', 'server_error', 'malformed']) {
  const svc = fakeService(mode);
  await assert.rejects(
    capabilitiesFor(svc).activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 4}),
    e => e && e.code === 'uncertain',
    mode + ' must surface an uncertain outcome');
  check(mode + ' -> code=uncertain', true);
  assert.equal(svc.casAttempts, 1, mode + ' must not replay the CAS');
  check(mode + ' spends exactly one CAS (no replay)', true);
  assert.deepEqual(
    JSON.parse(svc.store.get('v3admin:app:demo:state').toString()),
    {active: B, pointerVersion: 5},
    'the single write may still have committed; state must be reported, not guessed');
}

{
  const svc = fakeService('conflict');
  await assert.rejects(
    capabilitiesFor(svc).activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 4}),
    e => e && e.code === 'conflict',
    'definitive service rejection must stay a conflict, not become uncertain');
  check('definitive conflict is not mislabelled uncertain', true);
}

// ---------- B) controller: block replay until explicit reconciliation ----------
class Elem {
  constructor(tag) {
    this.tag = tag;
    this.children = [];
    this.listeners = new Map();
    this._text = '';
    this.className = '';
    this.hidden = false;
    this.type = '';
    this.disabled = false;
  }
  set textContent(v) { this._text = String(v); this.children = []; }
  get textContent() { return this._text + this.children.map(c => c.textContent).join(''); }
  append(...nodes) {
    for (const n of nodes) {
      if (n && n.__fragment) this.children.push(...n.children);
      else this.children.push(n);
    }
  }
  replaceChildren(...nodes) { this.children = []; this.append(...nodes); }
  addEventListener(ev, fn) {
    if (!this.listeners.has(ev)) this.listeners.set(ev, []);
    this.listeners.get(ev).push(fn);
  }
  async click() {
    if (this.disabled) return; // mirror the real DOM: disabled buttons do not fire
    for (const fn of this.listeners.get('click') || []) await fn({type: 'click'});
  }
  walk(out = []) {
    out.push(this);
    for (const c of this.children) if (c instanceof Elem) c.walk(out);
    return out;
  }
}

const host = new Elem('div');
const hosts = new Map([['host', host]]);
globalThis.document = {
  createElement: t => new Elem(t),
  createDocumentFragment: () => {
    const e = new Elem('#fragment');
    e.__fragment = true;
    return e;
  },
  createTextNode: t => {
    const e = new Elem('#text');
    e._text = String(t);
    return e;
  },
  getElementById: id => hosts.get(id) ?? null,
};
const previousConfirm = globalThis.confirm;
globalThis.confirm = () => true;
const statuses = [];

try {
  const svc = fakeService('lost_ack');
  const caps = capabilitiesFor(svc);
  let activateCalls = 0;
  const originalActivate = caps.activateRelease.bind(caps);
  caps.activateRelease = async (...args) => { activateCalls += 1; return originalActivate(...args); };

  const controller = createAppsController({
    hostId: 'host',
    capabilities: caps,
    status: msg => statuses.push(String(msg)),
    onInspectManifest: () => {},
  });
  await controller.refresh();
  const activateButton = () => host.walk().find(e => e.tag === 'button' && /^Activate/.test(e.textContent));
  check('initial render offers activation', !!activateButton());

  await activateButton().click();
  assert.equal(activateCalls, 1);
  assert.ok(statuses.some(s => /unknown/i.test(s) && /reconcil|refresh/i.test(s)),
    'status must explain the unknown outcome and reconciliation step');
  check('unknown outcome is surfaced in status', true);
  assert.equal(activateButton().disabled, true, 'activation must be blocked until reconciled');
  check('activation button disabled after uncertain outcome', true);

  await activateButton().click();
  assert.equal(activateCalls, 1, 'controller must not replay an uncertain activation');
  check('controller does not replay while outcome is unknown', true);

  await controller.refresh();  // explicit authoritative state refresh
  assert.equal(activateButton().disabled, false, 'explicit refresh must re-enable activation');
  check('explicit refresh clears the reconciliation guard', true);
  await activateButton().click();
  assert.equal(activateCalls, 2);
  check('a fresh activation is allowed after reconciliation', true);
} finally {
  globalThis.confirm = previousConfirm;
  delete globalThis.document;
}

console.log(`PASS uncertain activation reconciliation contract (${ok} checks)`);
