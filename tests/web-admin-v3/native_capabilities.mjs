#!/usr/bin/env node
// Deterministic in-memory proof that the semantic adapter translates UI
// operations into the compiled P1 admin ABI (9/10/11/12/13 + 14 for app IDs),
// never AppPut/AppGet (6/7), and keeps activation a single u64 revision CAS.
// This is a wire-contract test only: real compiled-stack qualification lives in
// tests/web-admin-v3/real_native_e2e/p1_admin_adapter.mjs.
import assert from 'node:assert/strict';
import {createHash, randomBytes} from 'node:crypto';

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
const activeRecord = s => {
  const b = Buffer.alloc(42);
  b[0] = 3; b[1] = s.live ? 1 : 0;
  Buffer.from(s.hash).copy(b, 2);
  b.writeBigUInt64LE(BigInt(s.revision), 34);
  return b;
};
function readBlob(b, at) {
  const len = b.readUInt32LE(at);
  return {data: b.subarray(at + 4, at + 4 + len), next: at + 4 + len};
}
function readId(b, at) {
  const len = b.readUInt16LE(at);
  return {data: b.subarray(at + 2, at + 2 + len), next: at + 2 + len};
}
function readPage(b, at) {
  const present = b[at]; at += 1;
  const after = present ? b.subarray(at, at + 32) : null;
  if (present) at += 32;
  const limit = b.readUInt16LE(at);
  return {after, limit};
}
function makePage(hashes, after, limit) {
  if (limit === 0 || limit > 64) return envelope(1, Buffer.from([1]));
  const sorted = [...hashes].map(h => Buffer.from(h)).sort((a, b) => a.compare(b));
  const cursor = after ? Buffer.from(after) : null;
  const start = cursor ? sorted.findIndex(h => h.compare(cursor) > 0) : 0;
  const from = start < 0 ? sorted.length : start;
  const selected = sorted.slice(from, from + limit);
  const body = Buffer.alloc(2 + selected.length * 32);
  body.writeUInt16LE(selected.length, 0);
  selected.forEach((h, i) => h.copy(body, 2 + i * 32));
  return envelope(0, body);
}

const {createNativeCapabilities} = await import(
  new URL('../../web-admin-v3/native_capabilities.js', import.meta.url).href);

// Minimal compiled-service model. Only admin opcodes are understood; any
// AppPut/AppGet (6/7) frame is recorded and denied as a contract violation.
function fakeService() {
  const programs = new Map();
  const opcodes = [];
  const casFrames = [];
  const handler = bytes => {
    const b = Buffer.from(bytes);
    const op = b[1];
    opcodes.push(op);
    assert.equal(b[0], 1, 'version byte');
    if (op === 6 || op === 7) return envelope(1, Buffer.from([1]));
    if (op === 15) {
      let at = 2;
      const opCap = readId(b, at); at = opCap.next;
      const admin = readId(b, at); at = admin.next;
      const code = readBlob(b, at).data;
      assert.equal(opCap.data.length, 32);
      const iid = sha(admin.data);
      const hash = sha(code);
      programs.set(iid.toString('hex'), {
        admin: Buffer.from(admin.data),
        sealed: new Set([hash.toString('hex')]),
        blobs: new Map([[hash.toString('hex'), Buffer.from(code)]]),
        active: {hash, revision: 1, live: true},
      });
      return envelope(0, iid);
    }
    if (op === 9) {
      let at = 2;
      const admin = readId(b, at); at = admin.next;
      const code = readBlob(b, at).data;
      const iid = sha(admin.data).toString('hex');
      const p = programs.get(iid);
      if (!p) return envelope(1, Buffer.from([8]));
      const hash = sha(code);
      p.blobs.set(hash.toString('hex'), Buffer.from(code));
      return envelope(0, hash);
    }
    if (op === 10) {
      let at = 2;
      const admin = readId(b, at); at = admin.next;
      const hash = b.subarray(at, at + 32);
      const p = programs.get(sha(admin.data).toString('hex'));
      if (!p) return envelope(1, Buffer.from([8]));
      p.sealed.add(hash.toString('hex'));
      return envelope(0);
    }
    if (op === 11) {
      casFrames.push(Buffer.from(b));
      let at = 2;
      const admin = readId(b, at); at = admin.next;
      const expected = Number(b.readBigUInt64LE(at)); at += 8;
      const target = b.subarray(at, at + 32);
      const p = programs.get(sha(admin.data).toString('hex'));
      if (!p || !p.active.live) return envelope(1, Buffer.from([8]));
      if (p.active.revision !== expected) return envelope(1, Buffer.from([10]));
      if (!p.sealed.has(target.toString('hex'))) return envelope(1, Buffer.from([8]));
      p.active = {hash: Buffer.from(target), revision: expected + 1, live: true};
      return envelope(0, activeRecord(p.active));
    }
    if (op === 12) {
      const iid = readId(b, 2).data;
      const p = programs.get(iid.toString('hex'));
      return p ? envelope(0, activeRecord(p.active)) : envelope(1, Buffer.from([8]));
    }
    if (op === 13) {
      let at = 2;
      const iid = readId(b, at).data; at = readId(b, 2).next;
      const page = readPage(b, at);
      const p = programs.get(iid.toString('hex'));
      return p
        ? makePage([...p.sealed].map(h => Buffer.from(h, 'hex')), page.after, page.limit)
        : envelope(1, Buffer.from([8]));
    }
    if (op === 14) {
      const page = readPage(b, 2);
      return makePage([...programs.keys()].map(h => Buffer.from(h, 'hex')), page.after, page.limit);
    }
    throw Error('unexpected opcode ' + op);
  };
  const fetchImpl = async (_url, init) => {
    const body = handler(init.body);
    return {status: 200, arrayBuffer: async () =>
      body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength)};
  };
  return {programs, opcodes, casFrames, fetchImpl};
}

let ok = 0;
const check = (name, cond, detail = '') => {
  assert.ok(cond, name + (detail ? ': ' + detail : ''));
  ok += 1;
  console.log('PASS ' + name + (detail ? ' :: ' + detail : ''));
};

const svc = fakeService();
const admin = sha('admin');
const iid = sha(admin);
const codeA = Buffer.from('native-A-' + randomBytes(8).toString('hex'));
const codeB = Buffer.from('native-B-' + randomBytes(8).toString('hex'));
const A = sha(codeA).toString('hex'), B = sha(codeB).toString('hex');
const app = iid.toString('hex');

// Register directly at the fake service (op 15 is out-of-band provisioning).
{
  const p = iid, h = sha(codeA);
  svc.programs.set(p.toString('hex'), {
    admin: Buffer.from(admin),
    sealed: new Set([h.toString('hex')]),
    blobs: new Map([[h.toString('hex'), Buffer.from(codeA)]]),
    active: {hash: h, revision: 1, live: true},
  });
}
const caps = createNativeCapabilities({
  endpoint: 'http://127.0.0.1:1/_skrynia/',
  readCap: app,
  adminCap: admin.toString('hex'),
}, svc.fetchImpl);

assert.deepEqual(await caps.listApps(), [app]);
check('listApps uses LIST_PROGRAMS (14), not AppGet', svc.opcodes.includes(14));
const state = await caps.getAppState(app);
check('getAppState GET_ACTIVE + LIST_VERSIONS (12/13)', svc.opcodes.includes(12) && svc.opcodes.includes(13));
check('getAppState reports all ready versions', state.versions.length === 1 && state.versions[0].id === A);
check('getAppState reports active hash + u64 revision', state.active === A && state.pointerVersion === 1);

const staged = await caps.stageRelease(app, codeB);
check('stageRelease uses STAGE (9) and returns sha256(code)', staged === B && svc.opcodes.includes(9));
check('staged-but-unsealed release is absent from ready history',
  (await caps.getAppState(app)).versions.every(v => v.id !== B));
const beforeUnsealed = svc.casFrames.length;
await assert.rejects(
  caps.activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 1}),
  e => e && e.code === 'invalid_state');
check('unsealed activation fails closed before the CAS', svc.casFrames.length === beforeUnsealed);
await caps.sealRelease(app, B);
check('sealRelease uses SEAL (10)', svc.opcodes.includes(10));
check('sealed release appears in history',
  (await caps.getAppState(app)).versions.map(v => v.id).sort().join() === [A, B].sort().join());

const manifest = await caps.getReleaseManifest(app, B);
check('getReleaseManifest is identity-only (no fabricated file metadata)',
  manifest.id === B && manifest.files === 0 && manifest.raw.origin === 'native-p1-ready-hash');

const switched = await caps.activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 1});
check('ACTIVATE (11) is one u64 revision CAS to a later version',
  switched.active === B && switched.pointerVersion === 2 && svc.casFrames.length === beforeUnsealed + 1);
const cas = svc.casFrames[svc.casFrames.length - 1];
check('ACTIVATE frame encodes admin cap, u64 revision and 32-byte target',
  cas[1] === 11 && cas.readBigUInt64LE(36) === 1n && cas.length === 2 + 34 + 8 + 32 &&
  Buffer.from(cas.subarray(4, 36)).equals(admin));
const back = await caps.activateRelease(app, A, {expectedActive: B, expectedPointerVersion: 2});
check('A-B-A rollback to an earlier release advances revision', back.active === A && back.pointerVersion === 3);
await assert.rejects(
  caps.activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 1}),
  e => e && e.code === 'conflict');
check('stale revision is rejected as conflict without a CAS', svc.casFrames.length === beforeUnsealed + 2);
await assert.rejects(
  caps.activateRelease(app, B, {expectedActive: A, expectedPointerVersion: 1}),
  e => e && e.code === 'conflict');
check('ABA replay of an old revision is rejected', svc.casFrames.length === beforeUnsealed + 2);
const after = await caps.getAppState(app);
check('conflicts left the durable pointer unchanged', after.active === A && after.pointerVersion === 3);
const foreign = sha('other-admin').toString('hex');
await assert.rejects(caps.activateRelease(foreign, B, {expectedActive: A, expectedPointerVersion: 3}),
  e => e && e.code === 'unauthorized');
check('activation without provisioned admin authority fails closed', true);
check('no AppPut/AppGet (6/7) opcode was ever sent',
  !svc.opcodes.includes(6) && !svc.opcodes.includes(7));

console.log(`PASS native P1 admin-opcode adapter contract (${ok} checks)`);
