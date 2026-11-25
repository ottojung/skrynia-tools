#!/usr/bin/env node
// Disposable REAL compiled-P1 capacity ceiling probe. Boots the pinned stack
// (kernel-release + dumb P0 + ordinary P1 + trusted edge gateway) over loopback
// and records the largest ready-marker count for which native LIST_VERSIONS
// still succeeds before the shared syscall callback budget is exhausted.
// No simulator, no production data, no deploy, no rebuild, no network egress.
import assert from 'node:assert/strict';
import { randomBytes } from 'node:crypto';
import { mkdtemp, rm, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import {
  PINS, putId, putBlob, decodeResponse, freePort, stopProc,
  sha256hex, sha256buf, hashFile, exists,
  startKernel, startGateway, gatewayFetch,
} from './real_native_e2e/run.mjs';

const ADAPTER = new URL('../../web-admin-v3/binary_transport.js', import.meta.url).href;
const { createBinaryPostTransport } = await import(ADAPTER);
const MAX = Number(process.env.PROBE_MAX ?? 260);
const stageOp = (a, c) => Buffer.concat([Buffer.from([1, 9]), putId(a), putBlob(c)]);
const sealOp = (a, h) => Buffer.concat([Buffer.from([1, 10]), putId(a), Buffer.from(h)]);
const listOp = (i, limit) => {
  const l = Buffer.alloc(2); l.writeUInt16LE(limit);
  const page = Buffer.concat([Buffer.from([0]), l]);
  return Buffer.concat([Buffer.from([1, 13]), putId(i), page]);
};
const registerOp = (o, a, c) => Buffer.concat([Buffer.from([1, 15]), putId(o), putId(a), putBlob(c)]);

const tmp = await mkdtemp(join(tmpdir(), 'p1-cap-probe-'));
const procs = [];
const pins = {};
try {
  for (const [name, p] of Object.entries(PINS)) {
    if (['kernel', 'p0', 'service', 'echo', 'gateway'].includes(name)) {
      pins[name] = (await exists(p.path)) ? await hashFile(p.path) : 'MISSING';
      assert.equal(pins[name], p.sha256, `${name} pin mismatch`);
    }
  }
  const token = randomBytes(32), opCap = randomBytes(32), adminCap = randomBytes(32);
  const iid = sha256buf(adminCap);
  const echo = await readFile(PINS.echo.path);
  const kernelPort = await freePort();
  const kernel = await startKernel({ base: join(tmp, 'kernel'), port: kernelPort, token, opCap, logs: 'kernel.log' });
  procs.push(kernel.proc);
  assert.equal(kernel.proc.exitCode, null, 'kernel boot');
  const edgePort = await freePort();
  const gw = await startGateway({ dir: join(tmp, 'edge'), port: edgePort, kernelPort, token,
    edgeCap: 'p1-cap-probe-' + randomBytes(12).toString('hex'), logs: 'gateway.log' });
  procs.push(gw.proc);
  assert.equal(gw.proc.exitCode, null, 'gateway boot');
  const t = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
  const send = async f => decodeResponse(await t.post(f));
  const reg = await send(registerOp(opCap, adminCap, echo));
  assert.equal(reg.status, 0, 'register');
  let lastOk = 1, firstFail = null, failErr = null;
  for (let n = 0; n < MAX; n += 1) {
    const code = Buffer.from(`cap-probe-${n}`);
    const h = sha256buf(code);
    const s = await send(stageOp(adminCap, code));
    assert.equal(s.status, 0, `stage ${n}`);
    const seal = await send(sealOp(adminCap, h));
    assert.equal(seal.status, 0, `seal ${n}`);
    const list = await send(listOp(iid, 1));
    if (list.status !== 0) {
      firstFail = n + 2; failErr = list.body[0];
      break;
    }
    lastOk = n + 2;
  }
  const out = { pins, maxProbe: MAX, lastListOkAtReadyCount: lastOk,
    firstListFailureAtReadyCount: firstFail, failureNativeErr: failErr,
    note: 'pinned P1 c7239fc5 full-scan listing; exact 1000 enumeration not possible on shipped pin' };
  await writeFile(join(tmp, 'ceiling.json'), JSON.stringify(out, null, 2));
  console.log(JSON.stringify(out, null, 2));
} finally {
  for (const p of procs.reverse()) await stopProc(p);
  await rm(tmp, { recursive: true, force: true });
}
