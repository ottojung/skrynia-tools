#!/usr/bin/env node
// Explicit, non-vacuous compiled-stack proof of the complete ordered
// 1000-version ready set with the paged-index ordinary P1.
//
// Drives real opcodes STAGE=9 / SEAL=10 / GET_ACTIVE=12 / LIST_VERSIONS=13 /
// REGISTER=15 as opaque bytes through the shipped binary POST adapter ->
// pinned trusted-edge gateway -> pinned release kernel -> pinned dumb P0 ->
// the byte-pinned compiled P1. No simulator, no v2 route, no production data.
//
// Checks: 1001 ready markers in exactly 16 pages of limit=64 (15 full + 1
// tail of 41), strictly ascending raw-byte keys, exact set equality with the
// hashes returned by STAGE (no gap, no duplicate), EXCLUSIVE tail cursors
// (each request after == previous page's last 32-byte key), empty terminal
// page, and native err 1 (BADREQ) for malformed/overlimit pages.
//
// All five consumed artifacts are SHA-256 byte-verified at startup; a missing
// or substituted pin is a hard failure, never a silent skip.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const KIT = process.env.SKRYNIA_KIT_DIR || '/workspace/skrynia-exact-kit-73d8fa1';
const setPin = (env, val) => { if (val) process.env[env] = val; };
setPin('SKRYNIA_KERNEL_BIN', process.env.KERNEL_BIN || join(KIT, 'kernel-release'));
setPin('SKRYNIA_KERNEL_SHA256', process.env.KERNEL_SHA || 'a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1');
setPin('SKRYNIA_P0_WASM', process.env.P0_WASM || join(KIT, 'root.wasm'));
setPin('SKRYNIA_P0_SHA256', process.env.P0_SHA || '44e845a4ca1bfcf6713078ad40d2ce7d387595b08351a5a882bbc38549780c95');
setPin('SKRYNIA_P1_WASM', process.env.P1_WASM);
setPin('SKRYNIA_P1_SHA256', process.env.P1_SHA);
setPin('SKRYNIA_ECHO_GUEST', process.env.ECHO_WASM || '/workspace/skrynia-v3-integration-kit/tests/guests/echo_guest.wasm');
setPin('SKRYNIA_ECHO_SHA256', process.env.ECHO_SHA || 'dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5');
setPin('SKRYNIA_EDGE_GATEWAY_BIN', process.env.GATEWAY_BIN || '/workspace/skrynia-recovery-along-20261121/.cargo-target/debug/skrynia-v3-trusted-edge-gateway');
setPin('SKRYNIA_EDGE_GATEWAY_SHA256', process.env.GATEWAY_SHA || '9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f');

const RUN = new URL('../../tests/web-admin-v3/real_native_e2e/run.mjs', import.meta.url).href;
const {
  PINS, putId, putBlob, decodeResponse, freePort, stopProc,
  sha256buf, sha256hex, hashFile,
  startKernel, startGateway, gatewayFetch,
} = await import(RUN);
const ADAPTER = new URL('../../web-admin-v3/binary_transport.js', import.meta.url).href;
const { createBinaryPostTransport } = await import(ADAPTER);

const LIMIT = 64, N = 1000, EXPECTED_PAGES = 16;
const stageOp = (a, c) => Buffer.concat([Buffer.from([1, 9]), putId(a), putBlob(c)]);
const sealOp = (a, h) => Buffer.concat([Buffer.from([1, 10]), putId(a), Buffer.from(h)]);
const listOp = (i, after, limit) => {
  const l = Buffer.alloc(2); l.writeUInt16LE(limit);
  const page = after ? Buffer.concat([Buffer.from([1]), Buffer.from(after), l])
                     : Buffer.concat([Buffer.from([0]), l]);
  return Buffer.concat([Buffer.from([1, 13]), putId(i), page]);
};
const parseList = b => {
  const n = b.readUInt16LE(0);
  assert.equal(b.length, 2 + n * 32, 'version page length');
  return Array.from({ length: n }, (_, i) => b.subarray(2 + i * 32, 2 + (i + 1) * 32));
};
const checks = [];
const rec = (name, ok, detail = '') => {
  checks.push({ name, ok: Boolean(ok), detail: String(detail) });
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
};

const pins = {};
for (const [name, p] of Object.entries(PINS)) {
  pins[name] = { path: p.path, expected: p.sha256, actual: await hashFile(p.path) };
  rec(`pin ${name} byte-verified`, pins[name].actual === pins[name].expected, pins[name].actual.slice(0, 12));
  assert.equal(pins[name].actual, pins[name].expected, `${name} pin mismatch`);
}

const tmp = await mkdtemp(join(tmpdir(), 'z21-paged-enum-'));
const procs = [];
try {
  const token = randomBytes(32), opCap = randomBytes(32), adminCap = randomBytes(32);
  const iid = sha256buf(adminCap);
  const echo = await readFile(PINS.echo.path);
  const hA = sha256buf(echo);
  const kPort = await freePort();
  const kernel = await startKernel({ base: join(tmp, 'kernel'), port: kPort, token, opCap, logs: 'kernel.log' });
  procs.push(kernel.proc);
  assert.equal(kernel.proc.exitCode, null, 'kernel boot');
  const gPort = await freePort();
  const gw = await startGateway({ dir: join(tmp, 'edge'), port: gPort, kernelPort: kPort, token,
    edgeCap: 'z21-paged-enum-' + randomBytes(12).toString('hex'), logs: 'gateway.log' });
  procs.push(gw.proc);
  assert.equal(gw.proc.exitCode, null, 'gateway boot');
  const t = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
  const send = async f => decodeResponse(await t.post(f));

  const reg = Buffer.concat([Buffer.from([1, 15]), putId(opCap), putId(adminCap), putBlob(echo)]);
  rec('REGISTER via real compiled stack', (await send(reg)).status === 0);
  const expected = [hA];
  let allStaged = true;
  for (let i = 0; i < N; i += 1) {
    const code = Buffer.from(`v-${String(i).padStart(4, '0')}`);
    const h = sha256buf(code);
    const s = await send(stageOp(adminCap, code));
    if (s.status !== 0 || !s.body.subarray(0, 32).equals(h)) allStaged = false;
    const d = await send(sealOp(adminCap, h));
    if (d.status !== 0) allStaged = false;
    expected.push(h);
  }
  rec(`STAGE+SEAL ${N} distinct versions over real opcodes`, allStaged);
  expected.sort(Buffer.compare);
  rec('1001 expected ready hashes distinct', new Set(expected.map(h => h.toString('hex'))).size === N + 1);

  const seen = [], sizes = [], cursors = [];
  let cursor = null, terminalErr = null;
  for (let guard = 0; guard < 64; guard += 1) {
    const d = await send(listOp(iid, cursor, LIMIT));
    if (d.status !== 0) { terminalErr = d.body[0]; break; }
    const page = parseList(d.body);
    sizes.push(page.length);
    cursors.push(cursor ? cursor.toString('hex') : null);
    if (page.length === 0) break;
    seen.push(...page);
    if (page.length < LIMIT) break;
    cursor = page[page.length - 1];
  }
  rec('LIST_VERSIONS all pages status OK', terminalErr === null, `err=${terminalErr}`);
  rec(`1001 ready markers enumerated`, seen.length === N + 1, `${seen.length}`);
  rec(`exactly ${EXPECTED_PAGES} pages of limit 64`, sizes.length === EXPECTED_PAGES &&
      sizes.slice(0, -1).every(n => n === LIMIT) && sizes[sizes.length - 1] === 41,
      sizes.join(','));
  const hexes = seen.map(h => h.toString('hex'));
  rec('strictly ascending raw-byte order (no reversal)', hexes.every((h, i) => i === 0 || Buffer.compare(seen[i - 1], seen[i]) < 0));
  rec('exact set equality with STAGE hashes (no gap/duplicate)',
      hexes.join(',') === expected.map(h => h.toString('hex')).join(','));
  rec('exclusive tail cursors equal previous page last key',
      cursors.slice(1).every((c, i) => c === seen[i * LIMIT + LIMIT - 1].toString('hex')));

  const tail = await send(listOp(iid, expected[N], LIMIT));
  rec('terminal page after last key is empty, not an error', tail.status === 0 && parseList(tail.body).length === 0);
  const bad31 = await send(listOp(iid, Buffer.alloc(31), LIMIT));
  rec('malformed 31-byte cursor -> native err 1', bad31.status === 1 && bad31.body[0] === 1);
  const bad33 = await send(listOp(iid, Buffer.alloc(33), LIMIT));
  rec('malformed 33-byte cursor -> native err 1', bad33.status === 1 && bad33.body[0] === 1);
  const over = await send(listOp(iid, null, 65));
  rec('overlimit page limit=65 -> native err 1', over.status === 1 && over.body[0] === 1);
  const zero = await send(listOp(iid, null, 0));
  rec('zero page limit -> native err 1', zero.status === 1 && zero.body[0] === 1);

  const out = {
    artifact: process.env.P1_WASM || '', p1_sha256: pins.service.actual,
    pins, checks, result_count: checks.length,
    pass_count: checks.filter(c => c.ok).length,
    fail_count: checks.filter(c => !c.ok).length,
    page_sizes: sizes,
    tail_cursors: cursors,
    all_hashes_sha256: sha256hex(Buffer.from(hexes.join(':'))),
    ALL_OK: checks.every(c => c.ok),
  };
  if (process.env.ENUM_EVIDENCE_OUT) {
    const { writeFile } = await import('node:fs/promises');
    await writeFile(process.env.ENUM_EVIDENCE_OUT, JSON.stringify(out, null, 2) + '\n', { mode: 0o600 });
  }
  console.log(JSON.stringify(out, null, 2));
  if (!out.ALL_OK) process.exitCode = 1;
} finally {
  for (const p of procs.reverse()) { try { await stopProc(p); } catch {} }
  await rm(tmp, { recursive: true, force: true });
}
