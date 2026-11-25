#!/usr/bin/env node
// Strict page/cursor validation for the shipped web-admin adapter. The fake
// service returns raw native page bytes we control; no compiled stack is used.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createNativeCapabilities} from '../../web-admin-v3/native_capabilities.js';

const sha = b => createHash('sha256').update(Buffer.from(b)).digest();
const PAGE_LIMIT = 64;

const envelope = (status, body = Buffer.alloc(0)) => {
  const h = Buffer.alloc(6);
  h[0] = 1; h[1] = status; h.writeUInt32LE(body.length, 2);
  return Buffer.concat([h, body]);
};
const pageBody = hashes => {
  const b = Buffer.alloc(2 + hashes.length * 32);
  b.writeUInt16LE(hashes.length, 0);
  hashes.forEach((h, i) => Buffer.from(h).copy(b, 2 + i * 32));
  return b;
};
const hashAt = n => {
  const b = Buffer.alloc(32);
  b.writeUInt32BE(n, 28);
  return b;
};
const PAGE = Array.from({length: PAGE_LIMIT}, (_, i) => hashAt(i + 1));

function capsFor(pages, op = 14) {
  let index = 0;
  const ops = [];
  const fetchImpl = async (_url, init) => {
    const request = new Uint8Array(init.body);
    assert.equal(request[0], 1, 'version byte');
    ops.push(request[1]);
    assert.equal(request[1], op, 'expected listing opcode');
    const hashes = index < pages.length ? pages[index++] : [];
    const body = envelope(0, pageBody(hashes));
    return {status: 200, arrayBuffer: async () =>
      body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength)};
  };
  const caps = createNativeCapabilities({
    endpoint: 'http://127.0.0.1:1/_skrynia/',
    readCap: '11'.repeat(32),
    adminCap: sha('admin').toString('hex'),
  }, fetchImpl);
  return {caps, ops};
}

async function rejectStrict(promise, pattern) {
  await assert.rejects(promise, err => {
    assert.equal(err.code, 'invalid_data', 'strict validation must fail closed as invalid_data');
    assert.match(err.message, pattern);
    return true;
  });
}

// Overlimit count is refused even though the declared byte length is coherent.
await rejectStrict(capsFor([[...PAGE, hashAt(9999)]]).caps.listApps(), /exceeds/);

// Unsorted and repeated hashes inside one page are malformed service output.
await rejectStrict(capsFor([[PAGE[1], PAGE[0]]]).caps.listApps(), /ascending/);
await rejectStrict(capsFor([[PAGE[0], PAGE[0]]]).caps.listApps(), /ascending/);

// A backwards or repeated cursor across pages is refused, not repaired.
await rejectStrict(capsFor([PAGE, [PAGE[PAGE.length - 1]]]).caps.listApps(),
  /backwards or repeated/);
await rejectStrict(capsFor([PAGE, [PAGE[PAGE.length - 2]]]).caps.listApps(),
  /backwards or repeated/);

// No silent partial result when the 4096-page guard is exhausted: the service
// keeps returning full strictly-ascending pages, so listApps must throw.
const endless = Array.from({length: 4096}, (_, p) =>
  Array.from({length: PAGE_LIMIT}, (_, i) => hashAt(p * PAGE_LIMIT + i + 1)));
await rejectStrict(capsFor(endless).caps.listApps(), /guard exhausted/);

// A well-formed multi-page listing is still accepted unchanged: the first page
// is full, the second advances strictly past its last cursor.
{
  const tail = [257, 258, 259, 260].map(hashAt);
  const {caps, ops} = capsFor([PAGE, tail]);
  const list = await caps.listApps();
  assert.deepEqual(list, [...PAGE, ...tail].map(h => h.toString('hex')));
  assert.ok(ops.length >= 2 && ops.every(op => op === 14));
}

console.log('strict page/cursor validation tests passed');
