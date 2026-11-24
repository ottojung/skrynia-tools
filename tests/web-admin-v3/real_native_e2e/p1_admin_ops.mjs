#!/usr/bin/env node
// Pinned, non-vacuous acceptance for the REAL ordinary-P1 native admin opcodes.
//
// This test does NOT simulate releases with AppPut/AppGet. It sends the dedicated
// native service opcodes STAGE(9)/SEAL(10)/ACTIVATE(11)/GET_ACTIVE(12)/
// LIST_VERSIONS(13) as opaque bytes through the shipped web-admin binary POST
// adapter -> pinned trusted edge gateway -> pinned kernel -> pinned dumb P0 ->
// pinned ordinary WASM service. The adapter defines no opcodes and P0 decodes
// none; the semantics are enforced by the compiled P1.
//
// Disposable: fresh random bearer/admin/operator capabilities and stage bytes per
// run, private temp dirs, 0600 token file, nothing secret logged or written to
// EVIDENCE. No CLI, no Cargo/Podman/WASM build, no network beyond 127.0.0.1,
// no production/TLS/DNS/real data.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { readFile, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

import {
  PINS, P0_FRAME_PIN, putId, putBlob, decodeResponse, rawKernelBody, framed,
  startKernel, startGateway, gatewayFetch, rawGatewayPost, freePort, stopProc,
  sha256hex, sha256buf, hashFile, exists,
} from './run.mjs';

const ADAPTER = new URL('../../../web-admin-v3/binary_transport.js', import.meta.url).href;
const { createBinaryPostTransport } = await import(ADAPTER);

const OP = { STAGE: 9, SEAL: 10, ACTIVATE: 11, GET_ACTIVE: 12, LIST_VERSIONS: 13 };
const ERR = { BADREQ: 1, IO: 7, NOTFOUND: 8, UNAUTH: 9, CONFLICT: 10 };

// Exact-byte allowlist for the ordinary P1 service role. The PR58 default is the
// old aa6333a pin; the PR55 acceptance pin is also admitted. Any other byte
// substitution fails the SHA-256 gate below. CI exact-head runs may supply
// SKRYNIA_P1_SHA256 for the freshly built service artifact.
const P1_PINS = {
  d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2:
    'old aa6333a cb-guest export-table P1 (PR58 run.mjs default)',
  '83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a':
    'PR55 ordinary P1 acceptance pin (source aa6ffd56 lineage)',
  ...(process.env.SKRYNIA_P1_SHA256
    ? {[process.env.SKRYNIA_P1_SHA256.toLowerCase()]: `exact-head P1 override ${process.env.SKRYNIA_P1_SHA256.toLowerCase().slice(0, 12)}`}
    : {}),
};

// ---- dedicated admin request encoders (mirror native-service wire ABI) ----
const stageOp = (admin, code) =>
  Buffer.concat([Buffer.from([1, OP.STAGE]), putId(admin), putBlob(code)]);
const sealOp = (admin, hash) =>
  Buffer.concat([Buffer.from([1, OP.SEAL]), putId(admin), Buffer.from(hash)]);
const activateOp = (admin, revision, target) => {
  const r = Buffer.alloc(8); r.writeBigUInt64LE(BigInt(revision));
  return Buffer.concat([Buffer.from([1, OP.ACTIVATE]), putId(admin), r, Buffer.from(target)]);
};
const getActiveOp = iid => Buffer.concat([Buffer.from([1, OP.GET_ACTIVE]), putId(iid)]);
const listVersionsOp = (iid, after = null, limit = 256) => {
  const l = Buffer.alloc(2); l.writeUInt16LE(limit);
  const page = after
    ? Buffer.concat([Buffer.from([1]), Buffer.from(after), l])
    : Buffer.concat([Buffer.from([0]), l]);
  return Buffer.concat([Buffer.from([1, OP.LIST_VERSIONS]), putId(iid), page]);
};

function parseActive(body) {
  assert.equal(body.length, 42, 'active record must be exactly 42 bytes');
  return {
    ver: body[0], live: body[1], hash: body.subarray(2, 34),
    revision: Number(body.readBigUInt64LE(34)),
  };
}
function parseList(body) {
  const n = body.readUInt16LE(0);
  assert.equal(body.length, 2 + n * 32, 'version list body length');
  return Array.from({ length: n }, (_, i) => body.subarray(2 + i * 32, 2 + (i + 1) * 32));
}
const eqBuf = (a, b) => Buffer.from(a).equals(Buffer.from(b));
const sortedHex = xs => xs.map(x => Buffer.from(x).toString('hex')).sort().join(',');

export async function main() {
  const results = [];
  const verifiedPins = {};
  const rec = (name, ok, detail = '') => {
    results.push({ name, ok: Boolean(ok), detail: String(detail) });
    console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
    return ok;
  };
  let allOk = false;
  const tmp = await mkdtemp(join(tmpdir(), 'p1-admin-ops-e2e-'));
  const procs = [];
  let token, tokenHex, verifierHex, edgeCap;
  try {
    // Pins are verified from bytes actually consumed, not from path names.
    for (const [name, a] of Object.entries(PINS)) {
      const got = (await exists(a.path)) ? await hashFile(a.path) : 'MISSING';
      const expected = name === 'service' && P1_PINS[got] ? got : a.sha256;
      const note = name === 'service' && P1_PINS[got] ? P1_PINS[got] : a.note;
      verifiedPins[name] = { path: a.path, sha256: got, note };
      rec(`pin ${name} sha256 ${expected.slice(0, 12)}`, got === expected, note);
      assert.equal(got, expected, `pinned ${name} sha256 mismatch`);
    }

    token = randomBytes(32);
    tokenHex = token.toString('hex');
    verifierHex = sha256hex(token);
    const opCap = randomBytes(32);
    const adminCap = randomBytes(32);
    const iid = sha256buf(adminCap);
    const echo = await readFile(PINS.echo.path);
    const codeHash = h => sha256buf(h);
    const hA = codeHash(echo);

    const kernelPort = await freePort();
    const kernel = await startKernel({ base: join(tmp, 'kernel'), port: kernelPort, token, opCap, logs: 'kernel.log' });
    procs.push(kernel.proc);
    rec('pinned kernel booted with P0 root + real P1', kernel.proc.exitCode === null, `127.0.0.1:${kernelPort}`);

    const edgePort = await freePort();
    edgeCap = 'p1-admin-e2e-' + randomBytes(16).toString('hex');
    const gw = await startGateway({ dir: join(tmp, 'edge'), port: edgePort, kernelPort, token, edgeCap, logs: 'gateway.log' });
    procs.push(gw.proc);
    rec('trusted edge gateway booted with server-only token', gw.proc.exitCode === null, gw.host);

    const transport = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
    const secrets = [tokenHex, verifierHex, adminCap.toString('hex'), opCap.toString('hex'), edgeCap];
    const noLeak = buf => {
      const hex = Buffer.from(buf).toString('hex');
      const s = Buffer.from(buf).toString('utf8');
      return !secrets.some(v => v && (hex.includes(v.toLowerCase()) || s.includes(v)));
    };
    const send = async frame => {
      const raw = await transport.post(frame);
      const d = decodeResponse(raw);
      return { ...d, leak: !noLeak(raw) };
    };
    const okOp = async (name, frame, pred = () => true) => {
      const d = await send(frame);
      rec(name, d.status === 0 && pred(d) && !d.leak,
        `native status ${d.status}, body ${d.body.length}B${d.leak ? ' SECRET LEAK' : ''}`);
      return d;
    };
    const denyOp = async (name, frame, code = null) => {
      const d = await send(frame);
      const good = d.status === 1 && !d.leak && (code === null || d.body[0] === code);
      rec(name, good, `native status ${d.status}, err ${d.body[0]}${d.leak ? ' SECRET LEAK' : ''}`);
      return d;
    };

    // ---- register A (initial active code; register seals its hash as ready) ----
    await okOp('R1 register private guest A via adapter',
      Buffer.concat([Buffer.from([1, 0x0f]), putId(opCap), putId(adminCap), putBlob(echo)]));

    const a = await okOp('S1 get_active A: ver=3 live=1 rev=1 hash=sha(code)',
      getActiveOp(iid), d => {
        const s = parseActive(d.body);
        return s.ver === 3 && s.live === 1 && s.revision === 1 && eqBuf(s.hash, hA);
      });
    const revA = parseActive(a.body).revision;

    const l1 = await okOp('L1 list-all-ready = [A]', listVersionsOp(iid), d => {
      const xs = parseList(d.body);
      return xs.length === 1 && eqBuf(xs[0], hA);
    });

    // ---- stage B (not yet sealed) ----
    const codeB = Buffer.concat([Buffer.from('p1-admin-e2e-B-'), randomBytes(16)]);
    const hB = codeHash(codeB);
    await okOp('T1 stage B returns sha256(codeB)', stageOp(adminCap, codeB), d => eqBuf(d.body, hB));

    // ---- no-ready-marker negatives ----
    await denyOp('N1 activate B BEFORE seal denied (no ready marker, err 8)',
      activateOp(adminCap, revA, hB), ERR.NOTFOUND);

    // ---- seal B ----
    await okOp('T2 seal B', sealOp(adminCap, hB), d => d.body.length === 0);

    // ---- list all ready + pagination ----
    await okOp('L2 list-all-ready = [A,B] sorted', listVersionsOp(iid), d => {
      const xs = parseList(d.body);
      return xs.length === 2 && sortedHex(xs) === sortedHex([hA, hB]);
    });
    const firstPage = await okOp('L3 list page limit=1 returns exactly one', listVersionsOp(iid, null, 1), d => {
      const xs = parseList(d.body); return xs.length === 1;
    });
    const firstHash = parseList(firstPage.body)[0];
    await okOp('L4 list after=first returns the remaining hash', listVersionsOp(iid, firstHash, 256), d => {
      const xs = parseList(d.body); return xs.length === 1 && !eqBuf(xs[0], firstHash) &&
        sortedHex([...xs, firstHash]) === sortedHex([hA, hB]);
    });

    // ---- A -> B -> A with monotonic revision CAS ----
    const b = await okOp('C1 activate A->B (expected rev=1) ok rev=2 hash=B',
      activateOp(adminCap, revA, hB), d => {
        const s = parseActive(d.body); return s.revision === 2 && eqBuf(s.hash, hB) && s.ver === 3 && s.live === 1;
      });
    const revB = parseActive(b.body).revision;
    await okOp('C2 get_active now B rev=2', getActiveOp(iid), d => {
      const s = parseActive(d.body); return s.revision === 2 && eqBuf(s.hash, hB);
    });
    await denyOp('C3 stale revision 1 denied (CAS conflict, err 10)',
      activateOp(adminCap, revA, hA), ERR.CONFLICT);
    const c = await okOp('C4 activate B->A (expected rev=2) ok rev=3 hash=A (A/B/A)',
      activateOp(adminCap, revB, hA), d => {
        const s = parseActive(d.body); return s.revision === 3 && eqBuf(s.hash, hA);
      });
    const revC = parseActive(c.body).revision;
    await okOp('C5 get_active back to A rev=3', getActiveOp(iid), d => {
      const s = parseActive(d.body); return s.revision === 3 && eqBuf(s.hash, hA);
    });
    await denyOp('C6 ABA replay of revision 1 denied (err 10)',
      activateOp(adminCap, 1, hB), ERR.CONFLICT);
    await denyOp('C7 future revision denied (err 10)',
      activateOp(adminCap, revC + 5, hB), ERR.CONFLICT);
    await denyOp('N2 random unsealed target denied with correct revision (err 8)',
      activateOp(adminCap, revC, randomBytes(32)), ERR.NOTFOUND);

    // ---- deliberately incorrect admin frames (malformed, fail closed) ----
    const revBuf = Buffer.alloc(8); revBuf.writeBigUInt64LE(BigInt(revC));
    await denyOp('M1 activate with 31-byte target denied (malformed frame)',
      Buffer.concat([Buffer.from([1, OP.ACTIVATE]), putId(adminCap), revBuf, hB.subarray(0, 31)]));
    await denyOp('M2 list_versions with limit=0 denied (malformed page)',
      Buffer.concat([Buffer.from([1, OP.LIST_VERSIONS]), putId(iid), Buffer.from([0, 0, 0])]));
    await denyOp('M3 stage with 31-byte admin cap denied (bad auth frame)',
      Buffer.concat([Buffer.from([1, OP.STAGE]), putId(adminCap.subarray(0, 31)), putBlob(codeB)]), ERR.UNAUTH);

    // ---- fail-closed gateway credential / origin negatives ----
    const benign = getActiveOp(iid);
    const gwNeg = [
      ['G1 wrong edge capability -> 401', { host: gw.host, origin: gw.origin, edgeCap: 'wrong', port: edgePort }, 401],
      ['G2 missing edge capability -> 401', { host: gw.host, origin: gw.origin, edgeCap: null, port: edgePort }, 401],
      ['G3 wrong origin -> 403', { host: gw.host, origin: 'http://evil.example', edgeCap, port: edgePort }, 403],
      ['G4 wrong host -> 403', { host: 'evil.example', origin: gw.origin, edgeCap, port: edgePort }, 403],
    ];
    for (const [name, opts, expected] of gwNeg) {
      const code = await rawGatewayPost({ ...opts, body: benign });
      rec(name, code === expected, `http ${code}`);
    }

    // ---- forged P0 framing denied by the real pinned P0; no secret leaked ----
    const payload = benign;
    const badFrames = [
      ['P1 wrong version 0x00', Buffer.concat([Buffer.from([0]), token, payload])],
      ['P2 wrong version 0x02', Buffer.concat([Buffer.from([2]), token, payload])],
      ['P3 wrong version 0xff', Buffer.concat([Buffer.from([0xff]), token, payload])],
      ['P4 single-bit wrong token', framed(Buffer.from(Uint8Array.from(token, (v, i) => i ? v : v ^ 1)), payload)],
      ['P5 zero token', framed(new Uint8Array(32), payload)],
      ['P6 digest-as-token', framed(sha256buf(token), payload)],
      ['P7 empty frame', Buffer.alloc(0)],
      ['P8 1-byte frame', Buffer.from([1])],
      ['P9 31-byte truncated header', Buffer.concat([Buffer.from([1]), token.subarray(0, 30)])],
      ['P10 32-byte truncated header', Buffer.concat([Buffer.from([1]), token.subarray(0, 31)])],
    ];
    for (const [name, body] of badFrames) {
      const r = await rawKernelBody(kernelPort, body);
      let denied = r.http >= 400;
      if (!denied) { try { denied = decodeResponse(r.body).status !== 0; } catch { denied = true; } }
      rec(name + ' denied', denied && noLeak(r.body), `http ${r.http}${noLeak(r.body) ? '' : ' SECRET LEAK'}`);
    }

    // ---- liveness / unchanged durable state after all denials ----
    await okOp('Z1 admin state unchanged after negatives (A rev=3)', getActiveOp(iid), d => {
      const s = parseActive(d.body); return s.revision === 3 && eqBuf(s.hash, hA);
    });

    const evidence = {
      kind: 'ordinary P1 native admin opcode E2E (raw dedicated opcodes 9/10/11/12/13)',
      p0_frame_pin: P0_FRAME_PIN,
      verified_at: new Date().toISOString(),
      node: process.version,
      git_head: process.env.SKRYNIA_GIT_HEAD || null,
      pins: verifiedPins,
      adapter: ADAPTER,
      adapter_endpoint: `http://${gw.host}/_skrynia/`,
      token_policy: 'random 32-byte bearer per run, 0600 temp file, never logged; no token/verifier/admin/edge capability in EVIDENCE',
      result_count: results.length,
      results,
      ALL_OK: results.every(r => r.ok),
    };
    if (process.env.E2E_EVIDENCE_OUT) {
      await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n', { mode: 0o600 });
    }
    console.log(JSON.stringify(evidence, null, 2));
    allOk = evidence.ALL_OK;
    console.log(allOk
      ? 'PASS P1 native admin opcodes: list-all-ready + A/B/A CAS + stale/ABA/unsealed denials over real gateway+kernel+P0+P1'
      : 'FAIL P1 native admin opcode E2E');
    if (!allOk) process.exitCode = 1;
  } finally {
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
