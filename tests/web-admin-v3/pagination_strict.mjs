#!/usr/bin/env node
// Strict page/cursor validation for the shipped web-admin adapter. The fake
// service returns raw native page bytes we control; no compiled stack is used.
//
// Wire contract confirmed from PR107 source (native-service):
//   * MAX_PAGE = 64: protocol/decoder/ops/admin/page.rs rejects limit 0 or >64
//     with DecodeErr::Op, mapped to native err 1 (BADREQ) by protocol/handle.rs.
//   * page response = u16 count || count * 32-byte index keys; the tail key is
//     exactly the last key (no synthetic marker). Keys are SHA-256 content
//     hashes in strictly ascending raw byte order; the after cursor is
//     exclusive (collect.rs skips k <= after).
// The mocks below generate content-addressed keys (sha256 of the staged code),
// not a synthetic counter, so repeated/backwards-cursor tests remain faithful.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createNativeCapabilities} from '../../web-admin-v3/native_capabilities.js';

const sha = b => createHash('sha256').update(Buffer.from(b)).digest();
const PAGE_LIMIT = 64;
const code = n => 'skrynia-ready-' + String(n).padStart(6, '0');
const hashAt = n => sha(code(n));                 // production-shaped tail key
const sortedHashes = n => Array.from({length: n}, (_, i) => hashAt(i + 1)).sort(Buffer.compare);
const hexOf = b => Buffer.from(b).toString('hex');

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
const responseFor = page => {
  const body = envelope(0, pageBody(page));
  return {status: 200, arrayBuffer: async () =>
    body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength)};
};
const session = {
  endpoint: 'http://127.0.0.1:1/_skrynia/',
  readCap: '11'.repeat(32),
  adminCap: hexOf(sha('admin')),
};

// Decode a LIST frame: version, op, u16 id length, id, after flag, optional
// 32-byte after key, u16 LE limit. Used to assert the client wire cap.
function parseFrame(buf) {
  const b = Buffer.from(buf);
  assert.equal(b[0], 1, 'version byte');
  const op = b[1];
  // LIST_VERSIONS(13) carries a u16-length app ID; LIST_PROGRAMS(14) does not.
  let at = 2;
  if (op === 13) at += 2 + b.readUInt16LE(at);
  const flag = b[at++];
  assert.ok(flag === 0 || flag === 1, 'after flag');
  let after = null;
  if (flag === 1) { after = b.subarray(at, at + 32); at += 32; }
  const limit = b.readUInt16LE(at);
  assert.equal(at + 2, b.length, 'frame has no trailing bytes');
  return {op, limit, after};
}

// Canned pages: lets us inject malformed overlimit/unsorted/backwards pages.
function capsForPages(pages, op = 14, observe = null) {
  let index = 0;
  const fetchImpl = async (_url, init) => {
    const frame = Buffer.from(init.body);
    observe?.(parseFrame(frame));
    const hashes = index < pages.length ? pages[index++] : [];
    return responseFor(hashes);
  };
  return {caps: createNativeCapabilities(session, fetchImpl)};
}

// Faithful service: returns content keys strictly greater than the exclusive
// cursor, at most PAGE_LIMIT of them. Models the compiled sorted paged index.
function capsForKeys(keys, op = 14, observe = null) {
  const fetchImpl = async (_url, init) => {
    const frame = parseFrame(init.body);
    assert.equal(frame.op, op, 'expected listing opcode');
    assert.equal(frame.limit, PAGE_LIMIT, 'client must request the capped page size');
    if (observe) observe(frame);
    const pool = frame.after
      ? keys.filter(k => Buffer.compare(k, frame.after) > 0)
      : keys;
    return responseFor(pool.slice(0, PAGE_LIMIT));
  };
  return {caps: createNativeCapabilities(session, fetchImpl)};
}

async function rejectStrict(promise, pattern) {
  await assert.rejects(promise, err => {
    assert.equal(err.code, 'invalid_data', 'strict validation must fail closed as invalid_data');
    assert.match(err.message, pattern);
    return true;
  });
}

// ---- malformed page / cursor fail-closed cases ----
const PAGE = sortedHashes(PAGE_LIMIT);

// Overlimit count is refused even though the declared byte length is coherent.
await rejectStrict(capsForPages([[...PAGE, hashAt(9999)]]).caps.listApps(), /exceeds/);

// Unsorted and repeated hashes inside one page are malformed service output.
await rejectStrict(capsForPages([[PAGE[1], PAGE[0]]]).caps.listApps(), /ascending/);
await rejectStrict(capsForPages([[PAGE[0], PAGE[0]]]).caps.listApps(), /ascending/);

// A backwards or repeated cursor across pages is refused, not repaired.
await rejectStrict(capsForPages([PAGE, [PAGE[PAGE.length - 1]]]).caps.listApps(),
  /backwards or repeated/);
await rejectStrict(capsForPages([PAGE, [PAGE[PAGE.length - 2]]]).caps.listApps(),
  /backwards or repeated/);

// No silent partial result when the 4096-page guard is exhausted: the service
// keeps returning full strictly-ascending pages, so listApps must throw.
const endlessKeys = sortedHashes(4096 * PAGE_LIMIT);
const endless = Array.from({length: 4096}, (_, p) =>
  endlessKeys.slice(p * PAGE_LIMIT, (p + 1) * PAGE_LIMIT));
await rejectStrict(capsForPages(endless).caps.listApps(), /guard exhausted/);

// ---- tail-key semantics: last 32-byte index key, strictly ascending, exclusive ----
{
  const frames = [];
  const keys = sortedHashes(1000);
  const {caps} = capsForKeys(keys, 14, f => frames.push(f));
  const list = await caps.listApps();
  assert.equal(list.length, 1000, 'all 1000 content hashes must be listed');
  assert.deepEqual(list, keys.map(hexOf), 'strict raw-byte ascending order preserved');
  assert.equal(frames.length, 16, '1000 keys / 64-page cap = 16 LIST_PROGRAMS requests');
  assert.ok(frames.every(f => f.limit === 64 && f.op === 14), 'limit=64 on every wire request');
  for (let i = 1; i < frames.length; i += 1) {
    assert.ok(frames[i].after, 'a tail cursor is sent for continuation pages');
    assert.equal(hexOf(frames[i].after), list[i * PAGE_LIMIT - 1],
      'tail key is exactly the last 32-byte key of the prior page (no marker)');
    assert.ok(Buffer.compare(frames[i].after, frames[i - 1].after || Buffer.alloc(32)) > 0,
      'tail cursor strictly advances');
  }
}

// A cursor equal to the last key (or beyond it) yields an empty terminal page,
// never an error and never a repeated key. Exactly-one-full-page case proves
// the exclusive cursor; the client stops on the short/empty terminal page.
{
  const keys = sortedHashes(PAGE_LIMIT);
  const frames = [];
  const {caps} = capsForKeys(keys, 14, f => frames.push(f));
  const list = await caps.listApps();
  assert.equal(list.length, PAGE_LIMIT, 'no duplicate of the last key');
  assert.equal(frames.length, 2, 'one full page then one empty terminal page');
  assert.equal(hexOf(frames[1].after), keys[keys.length - 1].toString('hex'));
}

console.log('strict page/cursor validation tests passed');
