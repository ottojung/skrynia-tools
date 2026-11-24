#!/usr/bin/env node
// Permanent pinned acceptance for the SHIPPED web-admin-v3 adapter speaking the
// real compiled P1 admin ABI. Unlike p1_admin_ops.mjs (raw frames), this drives
// createNativeCapabilities(): listApps -> LIST_PROGRAMS(14), getAppState ->
// GET_ACTIVE(12)+LIST_VERSIONS(13), stage/seal -> 9/10, activation -> ACTIVATE(11)
// with a u64 revision CAS, A->B->A, stale/ABA/unsealed denials.
//
// No simulator, no mock backend, no CLI, no Cargo/Podman/WASM build. Fresh
// random bearer/admin/operator capabilities and stage bytes per run, loopback
// only, servers always terminated, no secret in evidence or logs.

import assert from 'node:assert/strict';
import {createHash, randomBytes} from 'node:crypto';
import {mkdtemp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join, resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {setTimeout as delay} from 'node:timers/promises';

import {
  PINS, P0_FRAME_PIN, registerOp, decodeResponse, startKernel, startGateway,
  gatewayFetch, freePort, stopProc, sha256hex, sha256buf, hashFile, exists,
} from './run.mjs';

const ADAPTER = new URL('../../../web-admin-v3/native_capabilities.js', import.meta.url).href;
const TRANSPORT = new URL('../../../web-admin-v3/binary_transport.js', import.meta.url).href;
const {createNativeCapabilities} = await import(ADAPTER);
const {createBinaryPostTransport} = await import(TRANSPORT);

// Exact-byte allowlist; any substituted P1 fails these SHA-256 gates.
const P1_ALLOW = new Set([
  'd3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2',
  '83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a',
  ...(process.env.SKRYNIA_P1_SHA256 ? [process.env.SKRYNIA_P1_SHA256.toLowerCase()] : []),
]);

export async function main() {
  const results = [];
  const rec = (name, ok, detail = '') => {
    const pass = Boolean(ok);
    results.push({name, ok: pass, detail: String(detail)});
    console.log(`${pass ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
    return pass;
  };
  let allOk = false;
  const tmp = await mkdtemp(join(tmpdir(), 'p1-admin-adapter-'));
  const procs = [];
  try {
    for (const [name, pin] of Object.entries(PINS)) {
      const got = (await exists(pin.path)) ? await hashFile(pin.path) : 'MISSING';
      if (name === 'service') {
        rec('pinned P1 sha256 is allowlisted', P1_ALLOW.has(got), got.slice(0, 12));
        assert.ok(P1_ALLOW.has(got), 'P1 pin not allowlisted');
      } else {
        rec(`pin ${name} sha256 ${pin.sha256.slice(0, 12)}`, got === pin.sha256, got.slice(0, 12));
        assert.equal(got, pin.sha256, `pinned ${name} mismatch`);
      }
    }
    rec('P0 frame pin', P0_FRAME_PIN.startsWith('8a03330da0c9b1235'), P0_FRAME_PIN.slice(0, 12));

    const token = randomBytes(32);
    const tokenHex = token.toString('hex');
    const verifierHex = sha256hex(token);
    const opCap = randomBytes(32);
    const adminCap = randomBytes(32);
    const adminHex = adminCap.toString('hex');
    const iid = sha256buf(adminCap);
    const app = iid.toString('hex');
    const echo = await readFile(PINS.echo.path);
    const hA = sha256buf(echo).toString('hex');
    const codeB = Buffer.concat([Buffer.from('fo4-admin-B-'), randomBytes(16)]);
    const codeC = Buffer.concat([Buffer.from('fo4-admin-C-'), randomBytes(16)]);
    const hB = sha256buf(codeB).toString('hex');
    const hC = sha256buf(codeC).toString('hex');

    const kernelPort = await freePort();
    const kernel = await startKernel({base: join(tmp, 'kernel'), port: kernelPort,
      token, opCap, logs: 'kernel.log'});
    procs.push(kernel.proc);
    rec('pinned kernel booted', kernel.proc.exitCode === null, `127.0.0.1:${kernelPort}`);
    const edgePort = await freePort();
    const gw = await startGateway({dir: join(tmp, 'edge'), port: edgePort,
      kernelPort, token, edgeCap: 'fo4-adapter-' + randomBytes(16).toString('hex'), logs: 'gateway.log'});
    procs.push(gw.proc);
    rec('trusted gateway booted with server-only token', gw.proc.exitCode === null, gw.host);

    const endpoint = `http://${gw.host}/_skrynia/`;
    const rawFetch = gatewayFetch(gw);
    const secrets = [tokenHex, verifierHex, adminHex];
    let leaked = false;
    const noLeakFetch = async (url, init) => {
      const resp = await rawFetch(url, init);
      const buf = Buffer.from(await resp.arrayBuffer());
      const hex = buf.toString('hex');
      const text = buf.toString('utf8');
      if (secrets.some(s => hex.includes(s.toLowerCase()) || text.includes(s))) leaked = true;
      return {status: resp.status, arrayBuffer: async () => buf};
    };

    // Register A out of band (op 15 is provisioning, not part of the UI API).
    const raw = createBinaryPostTransport(endpoint, noLeakFetch);
    const registered = decodeResponse(await raw.post(
      registerOp(opCap, adminCap, echo)));
    rec('register initial ready release A', registered.status === 0 && !leaked,
      `status ${registered.status}`);
    assert.equal(registered.status, 0);

    const caps = createNativeCapabilities({endpoint, readCap: app, adminCap: adminHex}, noLeakFetch);

    assert.deepEqual(await caps.listApps(), [app]);
    rec('listApps discovers the app via LIST_PROGRAMS(14)', true, app.slice(0, 12));

    let state = await caps.getAppState(app);
    rec('getAppState GET_ACTIVE(12)+LIST_VERSIONS(13): A rev 1',
      state.active === hA && state.pointerVersion === 1 &&
      state.versions.length === 1 && state.versions[0].id === hA);

    const stagedB = await caps.stageRelease(app, codeB);
    rec('stageRelease STAGE(9) returns sha256(codeB)', stagedB === hB);
    await assert.rejects(
      caps.activateRelease(app, hB, {expectedActive: hA, expectedPointerVersion: 1}),
      e => e && e.code === 'invalid_state',
      'unsealed release must be rejected before the CAS');
    rec('unsealed activation fails closed before the CAS', true);
    await caps.sealRelease(app, hB);
    state = await caps.getAppState(app);
    rec('sealRelease SEAL(10) exposes both ready versions',
      state.versions.map(v => v.id).sort().join() === [hA, hB].sort().join());

    const manifest = await caps.getReleaseManifest(app, hB);
    rec('getReleaseManifest is identity-only (no invented file metadata)',
      manifest.id === hB && manifest.files === 0 && manifest.raw.origin === 'native-p1-ready-hash');

    const a2b = await caps.activateRelease(app, hB, {expectedActive: hA, expectedPointerVersion: 1});
    rec('ACTIVATE(11) A->B with u64 revision CAS', a2b.active === hB && a2b.pointerVersion === 2);
    state = await caps.getAppState(app);
    rec('get_active confirms B rev 2', state.active === hB && state.pointerVersion === 2);

    const b2a = await caps.activateRelease(app, hA, {expectedActive: hB, expectedPointerVersion: 2});
    rec('ACTIVATE(11) B->A completes A/B/A with rev 3', b2a.active === hA && b2a.pointerVersion === 3);
    await assert.rejects(
      caps.activateRelease(app, hB, {expectedActive: hA, expectedPointerVersion: 1}),
      e => e && e.code === 'conflict', 'stale revision must conflict');
    rec('stale revision is rejected as conflict', true);
    await assert.rejects(
      caps.activateRelease(app, hB, {expectedActive: hA, expectedPointerVersion: 1}),
      e => e && e.code === 'conflict', 'ABA replay must conflict');
    rec('ABA replay of an old revision is rejected', true);

    const stagedC = await caps.stageRelease(app, codeC);
    await assert.rejects(
      caps.activateRelease(app, stagedC, {expectedActive: hA, expectedPointerVersion: 3}),
      e => e && e.code === 'invalid_state', 'unsealed target with correct revision must fail closed');
    rec('random unsealed target denied with correct revision', stagedC === hC);

    state = await caps.getAppState(app);
    rec('liveness: state unchanged after all denials (A rev 3)',
      state.active === hA && state.pointerVersion === 3);
    rec('no bearer/admin capability leaked in any response', !leaked);

    const evidence = {
      kind: 'shipped web-admin-v3 adapter over real compiled P1 admin opcodes 9/10/11/12/13/14',
      p0_frame_pin: P0_FRAME_PIN,
      verified_at: new Date().toISOString(),
      node: process.version,
      git_head: process.env.SKRYNIA_GIT_HEAD || null,
      p1_sha256: await hashFile(PINS.service.path),
      adapter: ADAPTER,
      adapter_endpoint: endpoint,
      token_policy: 'fresh random 32-byte bearer and admin capability per run; never logged or written to EVIDENCE',
      result_count: results.length,
      results,
      ALL_OK: results.every(r => r.ok),
    };
    if (process.env.E2E_EVIDENCE_OUT) {
      await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n', {mode: 0o600});
    }
    allOk = evidence.ALL_OK;
    console.log(allOk
      ? 'PASS shipped adapter -> real P1: list-all + stage/seal + A/B/A CAS + stale/ABA/unsealed fail-closed'
      : 'FAIL shipped adapter real-P1 admin acceptance');
  } catch (e) {
    console.error('ERROR', e.stack || String(e));
  } finally {
    for (const p of procs) { try { await stopProc(p); } catch {} }
    await delay(200);
    if (allOk) await rm(tmp, {recursive: true, force: true});
    else console.error('preserved tmp for diagnosis:', tmp);
  }
  if (!allOk) process.exitCode = 1;
  return allOk;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
}
