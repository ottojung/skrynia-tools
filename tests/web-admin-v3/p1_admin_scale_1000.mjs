#!/usr/bin/env node
// Permanent, non-vacuous 1000-historical-version / restart-durability acceptance
// for the REAL ordinary nonzero-WASM P1 code registry over the exact pinned
// release stack. This is a test only: no CLI, no simulator, no AppPut/AppGet
// stand-in, no injected fake capability, no Cargo/Podman/WASM rebuild.
//
// Stack (SHA-256 verified from the exact bytes consumed at startup):
//   release kernel 806c172 build kernel-release  -> a1dc4119...
//   dumb P0 root.wasm 44e845a4...  (token gate + 33-byte strip, decodes no op)
//   ordinary P1 service.wasm c7239fc5... (opcodes 9/10/11/12/13/14 in WASM)
//   trusted edge gateway 9fa4df14... (server-only bearer, apex-owned, unmodified)
//
// The test drives the dedicated native opcodes STAGE=9/SEAL=10/ACTIVATE=11/
// GET_ACTIVE=12/LIST_VERSIONS=13 as opaque bytes through the shipped
// createBinaryPostTransport() adapter -> pinned gateway -> pinned kernel ->
// pinned dumb P0 -> pinned compiled P1. The adapter defines no opcodes and P0
// decodes none; every release semantic below is enforced by the compiled P1.
// Opcodes 6/7 (AppPut/AppGet) are NEVER sent.
//
// O(1) proof is black-box: the entire private-state tree under the kernel user
// dir is hashed before and after each activation. With 1001 sealed versions
// resident, an activation must change exactly ONE 42-byte active record and
// leave every code blob and sealed-ready marker byte-identical (no migration,
// no history scan in the write path). The wire frame is a fixed 76-byte
// request and a fixed 42-byte active record, independent of history size.
//
// Disposable: fresh random bearer/operator/admin capabilities and stage code
// per run, 0600 token file, private loopback-only ports, no secret in logs or
// EVIDENCE. No production/TLS/DNS/real data, no merge, no deploy.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import http from 'node:http';
import { mkdtemp, mkdir, readFile, readdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

// Pin paths/hashes are fixed here, not trusted from the caller's environment:
// a substituted artifact can only pass if its bytes hash to these values.
// A rebuilt P1 may be validated by supplying BOTH SKRYNIA_P1_OVERRIDE (path)
// and SKRYNIA_P1_OVERRIDE_SHA256 (exact 64-hex digest); bytes are still hashed
// at startup. The default remains the exact PR102 compiled-stack artifact.
const EXPECTED = {
  kernel: 'a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1',
  p0: '44e845a4ca1bfcf6713078ad40d2ce7d387595b08351a5a882bbc38549780c95',
  service: process.env.SKRYNIA_P1_OVERRIDE_SHA256 ||
    'c7239fc5bfd6977e7835eef461a75e274270d9e721d8a2d88121906b1b0c3a36',
  echo: 'dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5',
  gateway: '9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f',
};
const KIT = process.env.SKRYNIA_KIT_DIR || '/workspace/skrynia-exact-kit-73d8fa1';
const ECHO = process.env.SKRYNIA_ECHO_GUEST ||
  '/workspace/skrynia-v3-integration-kit/tests/guests/echo_guest.wasm';
const GATEWAY = process.env.SKRYNIA_EDGE_GATEWAY_BIN ||
  '/workspace/skrynia-recovery-along-20261121/.cargo-target/debug/skrynia-v3-trusted-edge-gateway';

process.env.SKRYNIA_KERNEL_BIN = join(KIT, 'kernel-release');
process.env.SKRYNIA_KERNEL_SHA256 = EXPECTED.kernel;
process.env.SKRYNIA_P0_WASM = join(KIT, 'root.wasm');
process.env.SKRYNIA_P0_SHA256 = EXPECTED.p0;
process.env.SKRYNIA_P1_WASM = process.env.SKRYNIA_P1_OVERRIDE || join(KIT, 'service.wasm');
process.env.SKRYNIA_P1_SHA256 = EXPECTED.service;
process.env.SKRYNIA_ECHO_GUEST = ECHO;
process.env.SKRYNIA_ECHO_SHA256 = EXPECTED.echo;
process.env.SKRYNIA_EDGE_GATEWAY_BIN = GATEWAY;
process.env.SKRYNIA_EDGE_GATEWAY_SHA256 = EXPECTED.gateway;

// run.mjs is imported dynamically so the pin env above is in effect first.
const {
  PINS, P0_FRAME_PIN, putId, putBlob, decodeResponse, rawKernelBody, framed,
  startKernel, startGateway, gatewayFetch, rawGatewayPost, freePort, stopProc,
  sha256hex, sha256buf, hashFile, exists,
} = await import('./real_native_e2e/run.mjs');

const TRANSPORT = new URL('../../web-admin-v3/binary_transport.js', import.meta.url).href;
const NATIVE = new URL('../../web-admin-v3/native_capabilities.js', import.meta.url).href;
const { createBinaryPostTransport } = await import(TRANSPORT);
const { createNativeCapabilities } = await import(NATIVE);

const OP = { STAGE: 9, SEAL: 10, ACTIVATE: 11, GET_ACTIVE: 12, LIST_VERSIONS: 13, REGISTER: 15 };
const ERR = { BADREQ: 1, IO: 7, NOTFOUND: 8, UNAUTH: 9, CONFLICT: 10 };
const NVERSIONS = 1000;
const PAGE_LIMIT = 256;

const stageOp = (admin, code) =>
  Buffer.concat([Buffer.from([1, OP.STAGE]), putId(admin), putBlob(code)]);
const sealOp = (admin, hash) =>
  Buffer.concat([Buffer.from([1, OP.SEAL]), putId(admin), Buffer.from(hash)]);
const activateOp = (admin, revision, target) => {
  const r = Buffer.alloc(8); r.writeBigUInt64LE(BigInt(revision));
  return Buffer.concat([Buffer.from([1, OP.ACTIVATE]), putId(admin), r, Buffer.from(target)]);
};
const getActiveOp = iid => Buffer.concat([Buffer.from([1, OP.GET_ACTIVE]), putId(iid)]);
const listVersionsOp = (iid, after = null, limit = PAGE_LIMIT) => {
  const l = Buffer.alloc(2); l.writeUInt16LE(limit);
  const page = after
    ? Buffer.concat([Buffer.from([1]), Buffer.from(after), l])
    : Buffer.concat([Buffer.from([0]), l]);
  return Buffer.concat([Buffer.from([1, OP.LIST_VERSIONS]), putId(iid), page]);
};
const registerOp = (opCap, adminCap, code) =>
  Buffer.concat([Buffer.from([1, OP.REGISTER]), putId(opCap), putId(adminCap), putBlob(code)]);

function parseActive(body) {
  assert.equal(body.length, 42, 'active record must be exactly 42 bytes (fixed-size pointer)');
  return {
    ver: body[0], live: body[1], hash: body.subarray(2, 34),
    revision: Number(body.readBigUInt64LE(34)),
  };
}
function parseList(body) {
  const n = body.readUInt16LE(0);
  assert.equal(body.length, 2 + n * 32, 'version-list body length');
  return Array.from({ length: n }, (_, i) => body.subarray(2 + i * 32, 2 + (i + 1) * 32));
}
const eqBuf = (a, b) => Buffer.from(a).equals(Buffer.from(b));
const sortedHex = xs => xs.map(x => Buffer.from(x).toString('hex')).sort().join(',');

// Full private-state fingerprint (path -> {size, sha256}) under the kernel user dir.
async function snapshotPrivate(us) {
  const out = {};
  async function walk(base, rel) {
    let entries = [];
    try { entries = (await readdir(join(base, rel), { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name)); } catch {}
    for (const e of entries) {
      const r = join(rel, e.name);
      if (e.isDirectory()) await walk(base, r);
      else if (e.isFile()) {
        const bytes = await readFile(join(base, r));
        out[r] = { size: bytes.length, sha256: sha256hex(bytes) };
      }
    }
  }
  await walk(join(us, 'programs'), '');
  return out;
}
function diffSnapshots(a, b) {
  const changed = [], added = [], removed = [];
  for (const k of Object.keys(a)) {
    if (!(k in b)) removed.push(k);
    else if (a[k].size !== b[k].size || a[k].sha256 !== b[k].sha256) changed.push(k);
  }
  for (const k of Object.keys(b)) if (!(k in a)) added.push(k);
  return { changed: changed.sort(), added: added.sort(), removed: removed.sort() };
}

// Loopback drop-after-commit proxy: forwards to the real gateway, lets the
// compiled P1 commit ACTIVATE, then returns an untrustworthy empty reply.
async function startDropProxy({ port, gatewayPort, gwHost, gwOrigin, edgeCap }) {
  const state = { drop: false, activateCount: 0 };
  const server = http.createServer(async (req, res) => {
    const chunks = [];
    for await (const c of req) chunks.push(c);
    const body = Buffer.concat(chunks);
    const isActivate = body.length > 2 && body[0] === 1 && body[1] === OP.ACTIVATE;
    if (isActivate) state.activateCount += 1;
    const up = http.request({ hostname: '127.0.0.1', port: gatewayPort, path: '/_skrynia/', method: 'POST',
      headers: { Host: gwHost, Origin: gwOrigin, 'X-Skrynia-Edge-Capability': edgeCap,
        'Content-Type': 'application/octet-stream', 'Content-Length': body.length } }, upstreamRes => {
      const out = [];
      upstreamRes.on('data', c => out.push(c));
      upstreamRes.on('end', () => {
        const buf = Buffer.concat(out);
        if (state.drop && isActivate) {
          res.writeHead(200, { 'Content-Type': 'application/octet-stream' }); res.end(); return;
        }
        res.writeHead(upstreamRes.statusCode, { 'Content-Type': 'application/octet-stream' });
        res.end(buf);
      });
    });
    up.on('error', () => { res.writeHead(502); res.end(); });
    up.end(body);
  });
  await new Promise(ok => server.listen(port, '127.0.0.1', ok));
  return { server, state, host: `127.0.0.1:${port}`, origin: `http://127.0.0.1:${port}` };
}

export async function main() {
  const results = [];
  const rec = (name, ok, detail = '') => {
    results.push({ name, ok: Boolean(ok), detail: String(detail) });
    console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
    return ok;
  };
  let allOk = false;
  const tmp = await mkdtemp(join(tmpdir(), 'p1-scale-1000-e2e-'));
  const procs = [];
  let token, edgeCap, transport, iid, iidHex, adminCap, opCap;
  let proxy = null;
  const evidence = {
    kind: 'ordinary P1 1000-historical-version + restart durability acceptance (real compiled stack)',
    node: process.version,
    verified_at: new Date().toISOString(),
    git_head: process.env.SKRYNIA_GIT_HEAD || null,
    p0_frame_pin: P0_FRAME_PIN,
    expected_pins: EXPECTED,
    env_paths: { kernel: PINS.kernel.path, p0: PINS.p0.path, service: PINS.service.path,
      echo: PINS.echo.path, gateway: PINS.gateway.path },
    token_policy: 'random 32-byte bearer per run, 0600 temp file, never logged; no secret in EVIDENCE',
    skipped: ['Chromium UI pagination (PAGE_LIMIT=256) - covered at shipped-adapter level here, not browser DOM',
      'Cargo/Rust tests, any WASM/kernel/gateway build, main/prod/DNS/TLS/deploy'],
  };
  // Source-only CI jobs do not provision the exact compiled kit. Skip there
  // rather than asserting a missing local path; pins stay byte-exact when
  // artifacts are present (compiled-stack job and developer runs).
  const missing = Object.keys(EXPECTED).filter(n => !PINS[n] || !PINS[n].path);
  const absent = [];
  for (const n of Object.keys(EXPECTED)) {
    if (!(await exists(PINS[n].path))) absent.push(n);
  }
  if (absent.length) {
    console.log(`SKIP 1000-version acceptance: exact compiled kit artifacts absent (${absent.join(', ')}); pins unchanged`);
    return;
  }
  try {
    // ---- exact pins / boot ----
    for (const [name, expected] of Object.entries(EXPECTED)) {
      const got = (await exists(PINS[name].path)) ? await hashFile(PINS[name].path) : 'MISSING';
      rec(`pin ${name} sha256 ${expected.slice(0, 12)} (exact kit byte)`, got === expected, got.slice(0, 12));
      assert.equal(got, expected, `pinned ${name} sha256 mismatch`);
      evidence[`pin_${name}`] = { path: PINS[name].path, sha256: got };
    }
    rec('P0 frame contract pin unchanged',
      P0_FRAME_PIN === '8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217');

    token = randomBytes(32);
    opCap = randomBytes(32);
    adminCap = randomBytes(32);
    iid = sha256buf(adminCap);
    iidHex = iid.toString('hex');
    const secrets = [token.toString('hex'), sha256hex(token), adminCap.toString('hex'),
      opCap.toString('hex')];
    const noLeak = buf => {
      const hex = Buffer.from(buf).toString('hex');
      const s = Buffer.from(buf).toString('utf8');
      return !secrets.some(v => v && (hex.includes(v.toLowerCase()) || s.includes(v)));
    };

    const kernelPort = await freePort();
    const base = join(tmp, 'kernel');
    let kernel = await startKernel({ base, port: kernelPort, token, opCap, logs: 'kernel.log' });
    procs.push(kernel.proc);
    rec('release kernel booted with dumb P0 root + real P1', kernel.proc.exitCode === null, `127.0.0.1:${kernelPort}`);

    const edgePort = await freePort();
    edgeCap = 'p1-scale-1000-' + randomBytes(16).toString('hex');
    secrets.push(edgeCap);
    let gw = await startGateway({ dir: join(tmp, 'edge'), port: edgePort, kernelPort, token, edgeCap, logs: 'gateway.log' });
    procs.push(gw.proc);
    rec('trusted edge gateway booted with server-only token', gw.proc.exitCode === null, gw.host);

    transport = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
    const send = async frame => {
      const raw = await transport.post(frame);
      return { ...decodeResponse(raw), leak: !noLeak(raw) };
    };
    const okOp = async (name, frame, pred = () => true) => {
      const d = await send(frame);
      const good = d.status === 0 && pred(d) && !d.leak;
      rec(name, good, `native status ${d.status}, body ${d.body.length}B${d.leak ? ' SECRET LEAK' : ''}`);
      assert.ok(good, name);
      return d;
    };
    const denyOp = async (name, frame, code = null) => {
      const d = await send(frame);
      const good = d.status === 1 && !d.leak && (code === null || d.body[0] === code);
      rec(name, good, `native status ${d.status}, err ${d.body[0]}${d.leak ? ' SECRET LEAK' : ''}`);
      assert.ok(good, name);
      return d;
    };

    // ---- per-object authority separation: operator install vs private admin vs public read ----
    const echo = await readFile(PINS.echo.path);
    const hA = sha256buf(echo);
    const hAHex = hA.toString('hex');
    await okOp('AUTH1 trusted operator installs object A (op 15)',
      registerOp(opCap, adminCap, echo));
    const a0 = await okOp('AUTH2 public invoke/read ID resolves A rev1 (op 12)', getActiveOp(iid), d => {
      const s = parseActive(d.body);
      return s.ver === 3 && s.live === 1 && s.revision === 1 && eqBuf(s.hash, hA);
    });
    let revision = parseActive(a0.body).revision;
    await denyOp('AUTH3 raw private admin cap is NOT the public read ID (err 8)',
      getActiveOp(adminCap), ERR.NOTFOUND);
    await denyOp('AUTH4 public read ID cannot write: STAGE denied (err 8)',
      stageOp(iid, echo), ERR.NOTFOUND);
    await denyOp('AUTH5 wrong operator install cap denied (err 9)',
      registerOp(randomBytes(32), adminCap, echo), ERR.UNAUTH);
    await denyOp('AUTH6 duplicate registration denied (err 10)',
      registerOp(opCap, adminCap, echo), ERR.CONFLICT);
    const foreignCap = randomBytes(32);
    await denyOp('AUTH7 foreign private cap has no authority over A (err 8)',
      stageOp(foreignCap, echo), ERR.NOTFOUND);
    const afterAuth = parseActive((await send(getActiveOp(iid))).body);
    rec('AUTH8 A unchanged after authority negatives',
      afterAuth.revision === 1 && eqBuf(afterAuth.hash, hA), `rev ${afterAuth.revision}`);

    // ---- stage + seal 1000 distinct content-addressed versions (real opcodes 9/10) ----
    const versions = [];
    const bulkStart = Date.now();
    let stageOk = 0, sealOk = 0;
    for (let i = 0; i < NVERSIONS; i += 1) {
      const code = Buffer.concat([Buffer.from(`scale-1000-v${i}-`), randomBytes(16)]);
      const h = sha256buf(code);
      const d = await send(stageOp(adminCap, code));
      if (d.status === 0 && eqBuf(d.body, h) && !d.leak) stageOk += 1;
      versions.push({ i, code, hash: h, hex: h.toString('hex') });
    }
    for (const v of versions) {
      const d = await send(sealOp(adminCap, v.hash));
      if (d.status === 0 && d.body.length === 0 && !d.leak) sealOk += 1;
    }
    const bulkMs = Date.now() - bulkStart;
    rec(`BULK1 STAGE returned sha256(code) for ${NVERSIONS}/${NVERSIONS} versions`,
      stageOk === NVERSIONS, `${stageOk}/${NVERSIONS} in ${bulkMs}ms`);
    rec(`BULK2 SEAL succeeded for ${NVERSIONS}/${NVERSIONS} versions`,
      sealOk === NVERSIONS, `${sealOk}/${NVERSIONS}`);
    const afterBulk = parseActive((await send(getActiveOp(iid))).body);
    rec('BULK3 active pointer untouched by 2000 staging/sealing ops',
      afterBulk.revision === 1 && eqBuf(afterBulk.hash, hA), `rev ${afterBulk.revision}`);
    evidence.bulk = { versions: NVERSIONS, stage_ok: stageOk, seal_ok: sealOk, elapsed_ms: bulkMs };

    // ---- required 1000-version listing gate + bounded pagination control ----
    // This exact compiled stack has an observed platform ceiling: a single
    // LIST_VERSIONS over >=199 ready markers returns native err 7 (IO), and a
    // small page limit cannot avoid it because list_ready enumerates and reads
    // every marker before slicing the page. Record that gate result exactly,
    // then continue the remaining 1000-scale checks using the hashes returned
    // by STAGE (GET_ACTIVE/ACTIVATE remain functional at that scale).
    const tryList = async id => {
      const out = [], sizes = [];
      let cursor = null, err = null;
      for (let guard = 0; guard < 4096; guard += 1) {
        const d = await send(listVersionsOp(id, cursor, PAGE_LIMIT));
        if (d.status !== 0) return { ok: false, count: out.length, pages: sizes.length, sizes, err: d.body[0] };
        const page = parseList(d.body);
        sizes.push(page.length);
        if (page.length === 0) break;
        out.push(...page);
        if (page.length < PAGE_LIMIT) break;
        cursor = page[page.length - 1];
      }
      return { ok: true, count: out.length, pages: sizes.length, sizes, hashes: out, err: null };
    };

    const scaleList = await tryList(iid);
    rec('SCALE-LIST all 1001 sealed ready versions listable (acceptance gate)',
      scaleList.ok && scaleList.count === NVERSIONS + 1,
      scaleList.ok ? `${scaleList.count} hashes in ${scaleList.pages} pages`
        : `BLOCKED native err ${scaleList.err} (platform ceiling below 1001 markers)`);

    // Bounded control object on the same real stack proves the pagination code
    // path and authority separation below the platform ceiling.
    const adminCapB = randomBytes(32);
    const iidB = sha256buf(adminCapB), iidBHex = iidB.toString('hex');
    await okOp('CTRL1 register bounded control object B (op 15)', registerOp(opCap, adminCapB, echo));
    const versionsB = [];
    for (let i = 0; i < 60; i += 1) {
      const code = Buffer.concat([Buffer.from(`bounded-b-${i}-`), randomBytes(8)]);
      const h = sha256buf(code);
      const sd = await send(stageOp(adminCapB, code));
      assert.equal(sd.status, 0, 'bounded stage');
      assert.ok(eqBuf(sd.body, h), 'bounded stage hash');
      const sl = await send(sealOp(adminCapB, h));
      assert.equal(sl.status, 0, 'bounded seal');
      versionsB.push(h);
    }
    const boundedList = await tryList(iidB);
    rec('CTRL2 LIST_VERSIONS lists all 61 bounded ready versions below the ceiling',
      boundedList.ok && boundedList.count === 61, `${boundedList.count} hashes`);
    const boundedFirst = await send(listVersionsOp(iidB, null, 1));
    const firstB = parseList(boundedFirst.body)[0];
    const boundedRest = await send(listVersionsOp(iidB, firstB, PAGE_LIMIT));
    const restB = parseList(boundedRest.body);
    const unionB = [firstB, ...restB].map(x => x.toString('hex')).sort();
    const allB = boundedList.ok ? boundedList.hashes.map(x => x.toString('hex')).sort() : [];
    rec('CTRL3 cursor pagination (limit=1 then after) covers the bounded history',
      boundedFirst.status === 0 && boundedRest.status === 0 && unionB.length === 61 &&
      unionB.join(',') === allB.join(','), `${unionB.length} hashes`);
    await denyOp('CTRL4 limit=0 rejected (err 1)',
      Buffer.concat([Buffer.from([1, OP.LIST_VERSIONS]), putId(iidB), Buffer.from([0, 0, 0])]), ERR.BADREQ);
    await denyOp('CTRL5 limit=257 rejected (err 1)',
      Buffer.concat([Buffer.from([1, OP.LIST_VERSIONS]), putId(iidB), Buffer.from([0, 1, 1])]), ERR.BADREQ);
    evidence.listing = {
      scale: { ok: scaleList.ok, count: scaleList.count, pages: scaleList.pages, err: scaleList.err },
      bounded: { count: boundedList.count, pages: boundedList.pages, sizes: boundedList.sizes },
      platform_ceiling: 'independent diagnostic: 198 ready markers OK, 199 FAIL (native err 7), for limit 1/16/256 alike',
    };

    // shipped adapter: the 1001-version history hits the same platform ceiling;
    // the bounded control proves the shipped pagination code path works below it.
    const caps = createNativeCapabilities(
      { endpoint: `http://${gw.host}/_skrynia/`, readCap: iidHex, adminCap: adminCap.toString('hex') },
      gatewayFetch(gw));
    let adapterScale = null;
    try { adapterScale = await caps.getAppState(iidHex); } catch (e) { adapterScale = { error: e.code || e.message }; }
    rec('ADAPTER-scale shipped adapter paginates 1001 ready versions (acceptance gate)',
      adapterScale && Array.isArray(adapterScale.versions) && adapterScale.versions.length === NVERSIONS + 1,
      adapterScale && adapterScale.versions ? `${adapterScale.versions.length}` : String(adapterScale.error));
    const capsB = createNativeCapabilities(
      { endpoint: `http://${gw.host}/_skrynia/`, readCap: iidBHex, adminCap: adminCapB.toString('hex') },
      gatewayFetch(gw));
    const capStateB = await capsB.getAppState(iidBHex);
    rec('ADAPTER-bounded shipped adapter paginates the bounded ready history',
      capStateB.versions.length === 61, `${capStateB.versions.length}`);
    rec('ADAPTER-bounded shipped adapter sees B sealed at rev1',
      capStateB.active === hAHex && capStateB.pointerVersion === 1,
      `${capStateB.active.slice(0, 12)} rev ${capStateB.pointerVersion}`);

    // ---- O(1) activation: full private-state diff across 1001 ready versions ----
    const activeRel = join('active', iidHex + '.bin');
    const beforeSnap = await snapshotPrivate(kernel.us);
    rec('O1 snapshot has fixed 42B pointer + full 1001-version A ready history resident',
      (beforeSnap[activeRel]?.size === 42) &&
      Object.keys(beforeSnap).filter(k => k.startsWith(`ready/${iidHex}/`) &&
        /^[0-9a-f]{64}\.ready$/.test(k.slice(`ready/${iidHex}/`.length))).length === NVERSIONS + 1 &&
      Object.keys(beforeSnap).filter(k => k.startsWith('blob/')).length >= NVERSIONS + 1,
      `${Object.keys(beforeSnap).length} files total (incl. bounded control object)`);

    const act1req = activateOp(adminCap, revision, versions[0].hash);
    rec('O2 ACTIVATE wire request is fixed 76 bytes', act1req.length === 76, `${act1req.length}B`);
    const t1 = Date.now();
    const act1 = await okOp('O3 activate earliest ready version rev1->2', act1req, d => {
      const s = parseActive(d.body);
      return s.revision === 2 && eqBuf(s.hash, versions[0].hash) && s.live;
    });
    const act1ms = Date.now() - t1;
    revision = parseActive(act1.body).revision;
    const afterSnap1 = await snapshotPrivate(kernel.us);
    const d1 = diffSnapshots(beforeSnap, afterSnap1);
    rec('O4 activation changed exactly one 42-byte active record',
      JSON.stringify(d1) === JSON.stringify({ changed: [activeRel], added: [], removed: [] }) &&
      afterSnap1[activeRel].size === 42, JSON.stringify(d1));
    rec('O5 all 1001 blobs and 1001 ready markers byte-identical after activation',
      Object.keys(beforeSnap).filter(k => k !== activeRel)
        .every(k => beforeSnap[k].sha256 === afterSnap1[k]?.sha256));

    const act2req = activateOp(adminCap, revision, versions[NVERSIONS - 1].hash);
    rec('O6 ACTIVATE request unchanged (76B) for last version', act2req.length === 76, `${act2req.length}B`);
    const t2 = Date.now();
    const act2 = await okOp('O7 activate latest ready version rev2->3 (O(1))', act2req, d => {
      const s = parseActive(d.body);
      return s.revision === 3 && eqBuf(s.hash, versions[NVERSIONS - 1].hash) && s.live;
    });
    const act2ms = Date.now() - t2;
    revision = parseActive(act2.body).revision;
    const afterSnap2 = await snapshotPrivate(kernel.us);
    const d2 = diffSnapshots(afterSnap1, afterSnap2);
    rec('O8 second activation again touched only the fixed active record',
      JSON.stringify(d2) === JSON.stringify({ changed: [activeRel], added: [], removed: [] }), JSON.stringify(d2));
    evidence.o1 = { activate_1_ms: act1ms, activate_2_ms: act2ms,
      request_bytes: 76, active_record_bytes: 42,
      diff_1: d1, diff_2: d2,
      files_unchanged: Object.keys(beforeSnap).length - 1 };

    // ---- CAS to arbitrary earlier/later versions; monotone u64 revision ----
    const act3 = await okOp('CAS1 activate middle v500 rev3->4 (arbitrary earlier)',
      activateOp(adminCap, revision, versions[500].hash), d => {
        const s = parseActive(d.body); return s.revision === 4 && eqBuf(s.hash, versions[500].hash);
      });
    revision = parseActive(act3.body).revision;
    const act4 = await okOp('CAS2 activate v0 rev4->5 (arbitrary later, A-B-A across history)',
      activateOp(adminCap, revision, versions[0].hash), d => {
        const s = parseActive(d.body); return s.revision === 5 && eqBuf(s.hash, versions[0].hash);
      });
    revision = parseActive(act4.body).revision;
    const live4 = parseActive((await send(getActiveOp(iid))).body);
    rec('CAS3 GET_ACTIVE resolves exact committed v0 at rev5',
      live4.revision === 5 && eqBuf(live4.hash, versions[0].hash), `rev ${live4.revision}`);
    evidence.activation_revisions = [1, 2, 3, 4, 5];

    // ---- fail-closed negatives at 1000-version scale ----
    await denyOp('NEG1 stale revision 1 denied (err 10)', activateOp(adminCap, 1, versions[1].hash), ERR.CONFLICT);
    await denyOp('NEG2 ABA replay of rev1 after 4 activations denied (err 10)',
      activateOp(adminCap, 1, versions[0].hash), ERR.CONFLICT);
    await denyOp('NEG3 future revision denied (err 10)',
      activateOp(adminCap, revision + 9, versions[1].hash), ERR.CONFLICT);
    await denyOp('NEG4 random unsealed 32-byte target denied with correct revision (err 8)',
      activateOp(adminCap, revision, randomBytes(32)), ERR.NOTFOUND);
    const unsealedCode = Buffer.concat([Buffer.from('scale-1000-unsealed-'), randomBytes(16)]);
    const unsealedHash = sha256buf(unsealedCode);
    const unsealedHex = unsealedHash.toString('hex');
    const stagedU = await okOp('NEG5 stage extra code without sealing (blob exists, not ready)',
      stageOp(adminCap, unsealedCode), d => eqBuf(d.body, unsealedHash));
    await denyOp('NEG6 staged-but-unsealed activation denied even at correct revision (err 8)',
      activateOp(adminCap, revision, unsealedHash), ERR.NOTFOUND);
    const forgedCap = randomBytes(32);
    await denyOp('NEG7 forged admin capability cannot activate (err 8)',
      activateOp(forgedCap, revision, versions[1].hash), ERR.NOTFOUND);
    const revBuf = Buffer.alloc(8); revBuf.writeBigUInt64LE(BigInt(revision));
    await denyOp('NEG8 malformed 31-byte activation target denied (err 1)',
      Buffer.concat([Buffer.from([1, OP.ACTIVATE]), putId(adminCap), revBuf, versions[1].hash.subarray(0, 31)]), ERR.BADREQ);
    await denyOp('NEG9 zero revision denied (err 9)',
      activateOp(adminCap, 0, versions[1].hash), ERR.UNAUTH);
    await denyOp('NEG10 seal with 31-byte admin cap denied (err 9)',
      Buffer.concat([Buffer.from([1, OP.SEAL]), putId(adminCap.subarray(0, 31)), versions[1].hash]), ERR.UNAUTH);

    // forged ready marker and corrupt blob fail closed (store is our own temp dir)
    const readyRel = join('ready', iidHex, versions[10].hex + '.ready');
    const markerOriginal = await readFile(join(kernel.us, 'programs', readyRel));
    await writeFile(join(kernel.us, 'programs', readyRel), Buffer.from('forged'));
    await denyOp('COR1 forged ready marker: SEAL denied (err 7)', sealOp(adminCap, versions[10].hash), ERR.IO);
    await denyOp('COR2 forged ready marker: ACTIVATE denied (err 7)',
      activateOp(adminCap, revision, versions[10].hash), ERR.IO);
    await writeFile(join(kernel.us, 'programs', readyRel), markerOriginal);
    rec('COR3 marker restored byte-exact', eqBuf(await readFile(join(kernel.us, 'programs', readyRel)), markerOriginal));

    const blobRel = join('blob', versions[11].hex + '.bin');
    const blobOriginal = await readFile(join(kernel.us, 'programs', blobRel));
    await writeFile(join(kernel.us, 'programs', blobRel), Buffer.from('corrupted'));
    await denyOp('COR4 corrupt code blob: SEAL denied (err 7)', sealOp(adminCap, versions[11].hash), ERR.IO);
    await writeFile(join(kernel.us, 'programs', blobRel), blobOriginal);
    rec('COR5 blob restored byte-exact', eqBuf(await readFile(join(kernel.us, 'programs', blobRel)), blobOriginal));

    // forged P0 framing + gateway auth boundary (no secret leaks)
    const benign = getActiveOp(iid);
    const badFrames = [
      ['P0F1 wrong version 0x00', Buffer.concat([Buffer.from([0]), token, benign])],
      ['P0F2 single-bit wrong token', framed(Buffer.from(Uint8Array.from(token, (v, i) => i ? v : v ^ 1)), benign)],
      ['P0F3 zero token', framed(new Uint8Array(32), benign)],
      ['P0F4 digest-as-token', framed(sha256buf(token), benign)],
      ['P0F5 truncated header (31B)', Buffer.concat([Buffer.from([1]), token.subarray(0, 30)])],
    ];
    for (const [name, body] of badFrames) {
      const r = await rawKernelBody(kernelPort, body);
      let denied = r.http >= 400;
      if (!denied) { try { denied = decodeResponse(r.body).status !== 0; } catch { denied = true; } }
      rec(name + ' denied fail-closed, no secret', denied && noLeak(r.body), `http ${r.http}`);
    }
    for (const [name, opts, expected] of [
      ['GW1 wrong edge capability -> 401', { host: gw.host, origin: gw.origin, edgeCap: 'wrong', port: edgePort }, 401],
      ['GW2 missing edge capability -> 401', { host: gw.host, origin: gw.origin, edgeCap: null, port: edgePort }, 401],
      ['GW3 wrong origin -> 403', { host: gw.host, origin: 'http://evil.example', edgeCap, port: edgePort }, 403],
      ['GW4 wrong host -> 403', { host: 'evil.example', origin: gw.origin, edgeCap, port: edgePort }, 403],
    ]) {
      const code = await rawGatewayPost({ ...opts, body: benign });
      rec(name, code === expected, `http ${code}`);
    }
    const liveNeg = parseActive((await send(getActiveOp(iid))).body);
    rec('NEG11 liveness: exact committed state intact after all denials',
      liveNeg.revision === 5 && eqBuf(liveNeg.hash, versions[0].hash), `rev ${liveNeg.revision}`);

    // ---- restart durability at 1000-version scale ----
    const beforeRestart = await snapshotPrivate(kernel.us);
    evidence.private_state_before_restart = {
      file_count: Object.keys(beforeRestart).length,
      active: beforeRestart[activeRel],
      digest_of_all_paths_and_hashes: sha256hex(Buffer.from(JSON.stringify(beforeRestart))),
    };
    await stopProc(gw.proc); procs.splice(procs.indexOf(gw.proc), 1);
    await stopProc(kernel.proc); procs.splice(procs.indexOf(kernel.proc), 1);
    await delay(300);
    rec('RST1 killed release kernel + gateway', true);

    const kernelPort2 = await freePort();
    kernel = await startKernel({ base, port: kernelPort2, token, opCap, logs: 'kernel-restart.log' });
    procs.push(kernel.proc);
    const edgePort2 = await freePort();
    gw = await startGateway({ dir: join(tmp, 'edge-restart'), port: edgePort2, kernelPort: kernelPort2,
      token, edgeCap, logs: 'gateway-restart.log' });
    procs.push(gw.proc);
    rec('RST2 restarted on the same private-state dirs',
      kernel.proc.exitCode === null && gw.proc.exitCode === null,
      `kernel ${kernelPort2}, gateway ${edgePort2}`);

    const afterRestart = await snapshotPrivate(kernel.us);
    rec('RST3 private state byte-identical across kill/restart (2003 files)',
      JSON.stringify(afterRestart) === JSON.stringify(beforeRestart),
      `${Object.keys(afterRestart).length} files`);
    evidence.private_state_after_restart = {
      file_count: Object.keys(afterRestart).length,
      digest_of_all_paths_and_hashes: sha256hex(Buffer.from(JSON.stringify(afterRestart))),
    };

    transport = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
    const liveR = parseActive((await send(getActiveOp(iid))).body);
    rec('RST4 active pointer resolves to exact committed v0 rev5 after restart',
      liveR.revision === 5 && eqBuf(liveR.hash, versions[0].hash), `rev ${liveR.revision}`);
    const listR = await tryList(iid);
    rec('RST5 all 1001 ready versions survive restart and stay listable (acceptance gate)',
      listR.ok && listR.count === NVERSIONS + 1,
      listR.ok ? `${listR.count}` : `BLOCKED native err ${listR.err} (same platform ceiling)`);
    await denyOp('RST6 uncommitted staged-but-unsealed code still not activatable (err 8)',
      activateOp(adminCap, 5, unsealedHash), ERR.NOTFOUND);
    await denyOp('RST7 stale CAS after restart denied (err 10)',
      activateOp(adminCap, 4, versions[1].hash), ERR.CONFLICT);
    const rstAct = await okOp('RST8 post-restart activation continues with next revision rev5->6',
      activateOp(adminCap, revision, versions[NVERSIONS - 1].hash), d => {
        const s = parseActive(d.body); return s.revision === 6 && eqBuf(s.hash, versions[NVERSIONS - 1].hash);
      });
    revision = parseActive(rstAct.body).revision;

    // ---- lost ack through the shipped adapter after restart ----
    // The 1000-version object cannot be paginated by the shipped adapter on
    // this stack (same platform ceiling), so drive this lost-ack/reconciliation
    // check on the bounded control object, whose history the adapter can list.
    const liveB = parseActive((await send(getActiveOp(iidB))).body);
    rec('ACK0 bounded control object survived restart at rev1',
      liveB.revision === 1 && eqBuf(liveB.hash, hA), `rev ${liveB.revision}`);
    const proxyPort = await freePort();
    proxy = await startDropProxy({ port: proxyPort, gatewayPort: edgePort2, gwHost: gw.host,
      gwOrigin: gw.origin, edgeCap });
    const dropCaps = createNativeCapabilities(
      { endpoint: proxy.origin + '/_skrynia/', readCap: iidBHex, adminCap: adminCapB.toString('hex') },
      gatewayFetch({ host: proxy.host, origin: proxy.origin, edgeCap }));
    proxy.state.drop = true;
    let uncertain = null;
    try {
      await dropCaps.activateRelease(iidBHex, versionsB[0].toString('hex'), { expectedPointerVersion: 1 });
    } catch (e) { uncertain = e; }
    rec('ACK1 shipped adapter reports uncertain on lost reply (no blind replay)',
      uncertain && uncertain.code === 'uncertain', uncertain ? uncertain.code : 'no error');
    const committed = parseActive((await send(getActiveOp(iidB))).body);
    rec('ACK2 drop-after-commit really committed rev1->2 on the compiled P1',
      committed.revision === 2 && eqBuf(committed.hash, versionsB[0]), `rev ${committed.revision}`);
    rec('ACK3 exactly one CAS spent (proxy saw 1 ACTIVATE, no replay)',
      proxy.state.activateCount === 1, `${proxy.state.activateCount}`);
    proxy.state.drop = false;
    const reconciled = await dropCaps.getAppState(iidBHex);
    rec('ACK4 explicit state refresh reconciles to exact rev2',
      reconciled.pointerVersion === 2 && reconciled.active === versionsB[0].toString('hex'),
      `rev ${reconciled.pointerVersion}`);
    const cont = await dropCaps.activateRelease(iidBHex, versionsB[1].toString('hex'), { expectedPointerVersion: 2 });
    rec('ACK5 post-restart CAS continues rev2->3 via shipped adapter',
      cont.pointerVersion === 3 && cont.active === versionsB[1].toString('hex'), JSON.stringify(cont));
    evidence.final = {
      scale_object: { revision, active: versions[NVERSIONS - 1].hex },
      bounded_object: { revision: cont.pointerVersion, active: cont.active },
      activate_requests_seen_by_proxy: proxy.state.activateCount,
    };

    allOk = results.every(r => r.ok);
    evidence.results = results;
    evidence.result_count = results.length;
    evidence.pass_count = results.filter(r => r.ok).length;
    evidence.fail_count = results.filter(r => !r.ok).length;
    evidence.ALL_OK = allOk;
    if (process.env.E2E_EVIDENCE_OUT) {
      await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n', { mode: 0o600 });
    }
    console.log(JSON.stringify(evidence, null, 2));
    console.log(allOk
      ? `PASS 1000-version + restart acceptance: ${evidence.pass_count}/${results.length} checks (DEV acceptance evidence only)`
      : `FAIL 1000-version acceptance: ${evidence.fail_count}/${results.length} checks failed`);
    if (!allOk) process.exitCode = 1;
  } finally {
    try { if (proxy) await new Promise(ok => proxy.server.close(ok)); } catch {}
    for (const p of procs) { try { await stopProc(p); } catch {} }
    await delay(200);
    if (allOk) await rm(tmp, { recursive: true, force: true });
    else console.error('preserved tmp for diagnosis:', tmp);
  }
  return allOk;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
}
