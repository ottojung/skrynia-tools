#!/usr/bin/env node
// Real native HTTP end-to-end qualification for the web-admin-v3 proxy-only
// Program 0 adapter. NO simulator, NO mock backend, NO fake adapter: this boots
// the pinned compiled kernel + dumb P0 + ordinary WASM service + trusted edge
// gateway, then drives the shipped web-admin binary transport over real HTTP.
//
// Proves two independent properties:
//   A) createBinaryPostTransport() -> trusted gateway -> kernel -> P0 (auth +
//      33-byte strip) -> ordinary service -> response -> adapter, byte-exact.
//   B) Forged P0 framing (wrong version, truncated header, wrong/zero/digest
//      token) is denied by the REAL pinned P0 at the kernel HTTP boundary.
//
// Disposable only: random 32-byte bearer generated per run, stored 0600 in a
// temp dir, never logged, never in public assets, URLs or this source file.
// No kernel/gateway source is modified; no deployment, TLS or DNS change.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import net from 'node:net';
import http from 'node:http';
import { mkdtemp, mkdir, readFile, writeFile, rm, stat } from 'node:fs/promises';
import * as fsn from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const ADAPTER = new URL('../../../web-admin-v3/binary_transport.js', import.meta.url).href;
const { createBinaryPostTransport } = await import(ADAPTER);

// Exact compiled-artifact pins (SHA-256 of bytes, verified at startup).
// For CI/exact-head runs, override path+sha256 via env; historical defaults are kept.
const pin = (envPath, envSha, dfltPath, dfltSha, note) => ({
  path: process.env[envPath] || dfltPath,
  sha256: (process.env[envSha] || dfltSha).toLowerCase(),
  note: process.env[envSha] ? `${note} (sha256 overridden by ${envSha})` : note,
});
export const PINS = {
  kernel: pin('SKRYNIA_KERNEL_BIN', 'SKRYNIA_KERNEL_SHA256',
    '/workspace/skrynia-v3-integration-kit/wasm-build-record/main-secure-pr122-20261121/skrynia-server',
    '8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f',
    'secure PR122 kernel, source c39bcbf1'),
  p0: pin('SKRYNIA_P0_WASM', 'SKRYNIA_P0_SHA256',
    '/workspace/skrynia-v3-integration-kit/artifacts/skrynia_program0_pr53_clean_rebuilt_d57ab27a.wasm',
    'd57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa',
    'clean dumb P0 PR53, source fdf0d027'),
  service: pin('SKRYNIA_P1_WASM', 'SKRYNIA_P1_SHA256',
    '/workspace/skrynia-v3-integration-kit/artifacts/skrynia_native_service_aa6333a_export_table_d3686527bbf29070.wasm',
    'd3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2',
    'ordinary nonzero WASM service, aa6333a cb-guest export table'),
  echo: pin('SKRYNIA_ECHO_GUEST', 'SKRYNIA_ECHO_SHA256',
    '/workspace/skrynia-v3-integration-kit/tests/guests/echo_guest.wasm',
    'dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5',
    'ordinary private guest used only to exercise service register/invoke'),
  gateway: pin('SKRYNIA_EDGE_GATEWAY_BIN', 'SKRYNIA_EDGE_GATEWAY_SHA256',
    '/workspace/skrynia-recovery-along-20261121/.cargo-target/debug/skrynia-v3-trusted-edge-gateway',
    '9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f',
    'trusted edge gateway, source commit 71773b52'),
};
export const P0_FRAME_PIN = '8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217';

const results = [];
function rec(name, ok, detail = '') {
  results.push({ name, ok: Boolean(ok), detail: String(detail) });
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
  return ok;
}
export const sha256hex = b => createHash('sha256').update(b).digest('hex');
export const sha256buf = b => createHash('sha256').update(b).digest();
export async function hashFile(p) { return sha256hex(await readFile(p)); }
export async function exists(p) { try { await stat(p); return true; } catch { return false; } }

export async function freePort() {
  return await new Promise((ok, no) => {
    const s = createServer();
    s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(e => e ? no(e) : ok(p)); });
  });
}
export async function waitTcp(port, proc, logPath, tries = 250) {
  for (let i = 0; i < tries; i += 1) {
    if (proc.exitCode !== null) {
      throw new Error('process died rc=' + proc.exitCode + ' tail=' + (await readFile(logPath, 'utf8')).slice(-800));
    }
    const up = await new Promise(res => {
      const s = net.createConnection({ host: '127.0.0.1', port }, () => { s.destroy(); res(true); });
      s.on('error', () => res(false)); s.setTimeout(200, () => { s.destroy(); res(false); });
    });
    if (up) return;
    await delay(100);
  }
  throw new Error('timeout waiting for tcp ' + port + ' tail=' + (await readFile(logPath, 'utf8')).slice(-800));
}
export async function stopProc(proc) {
  if (!proc || proc.exitCode !== null) return;
  proc.kill('SIGTERM');
  for (let i = 0; i < 50; i += 1) { if (proc.exitCode !== null) return; await delay(100); }
  try { proc.kill('SIGKILL'); } catch {}
}

// ---- native wire helpers (mirror the P1 ordinary service ABI; P0 decodes none) ----
export const putId = b => { const x = Buffer.from(b); const h = Buffer.alloc(2); h.writeUInt16LE(x.length); return Buffer.concat([h, x]); };
export const putBlob = b => { const x = Buffer.from(b); const h = Buffer.alloc(4); h.writeUInt32LE(x.length); return Buffer.concat([h, x]); };
export const registerOp = (opCap, adminCap, wasm) => Buffer.concat([Buffer.from([0x01, 0x0f]), putId(opCap), putId(adminCap), putBlob(wasm)]);
export const lookupOp = iid => Buffer.concat([Buffer.from([0x01, 0x04]), putId(iid)]);
export const invokeOp = (iid, payload) => Buffer.concat([Buffer.from([0x01, 0x03]), putId(iid), putBlob(payload)]);
export const appPutOp = (adminCap, key, value) => Buffer.concat([
  Buffer.from([0x01, 0x06]), putId(adminCap), putBlob(key),
  Buffer.from([0]), // expect-absent precondition
  putBlob(value),
]);
export const appCasOp = (adminCap, key, expectedHash, value) => Buffer.concat([
  Buffer.from([0x01, 0x06]), putId(adminCap), putBlob(key),
  Buffer.from([1]), Buffer.from(expectedHash), putBlob(value),
]);
export const appGetOp = (iid, key) => Buffer.concat([Buffer.from([0x01, 0x07]), putId(iid), putBlob(key)]);

// Kernel response envelope: 0x01(version) || status || u32 LE length || body.
export function decodeResponse(buf) {
  const b = Buffer.from(buf);
  if (b.length < 6 || b[0] !== 1) throw new Error('bad native response framing: ' + b.subarray(0, 16).toString('hex'));
  const len = b.readUInt32LE(2);
  if (6 + len !== b.length) throw new Error('bad native response length: ' + len + ' vs ' + (b.length - 6));
  return { status: b[1], body: b.subarray(6) };
}

// Direct kernel POST with an arbitrary full body (used only for forged frames).
export async function rawKernelBody(port, body, timeoutMs = 15000) {
  return await new Promise((ok, no) => {
    const req = http.request({ host: '127.0.0.1', port, path: '/', method: 'POST',
      headers: { 'Content-Type': 'application/octet-stream', 'Content-Length': body.length } }, res => {
      const chunks = []; res.on('data', c => chunks.push(c)); res.on('end', () => ok({ http: res.statusCode, body: Buffer.concat(chunks) }));
    });
    req.on('error', no); req.setTimeout(timeoutMs, () => req.destroy(new Error('kernel timeout')));
    req.end(Buffer.from(body));
  });
}
export const framed = (token, payload) => Buffer.concat([Buffer.from([0x01]), Buffer.from(token), Buffer.from(payload)]);

export async function startKernel({ base, port, token, opCap, logs }) {
  const op = join(base, 'operator'), us = join(base, 'user');
  await mkdir(join(op, 'security'), { recursive: true });
  await mkdir(join(op, 'programs'), { recursive: true });
  for (const s of ['active', 'blob', 'ready', 'appdata']) await mkdir(join(us, 'programs', s), { recursive: true });
  await writeFile(join(op, 'security', 'program0.sha256'), sha256buf(token), { mode: 0o600 });
  await writeFile(join(op, 'programs', 'root.wasm'), await readFile(PINS.service.path));
  await writeFile(join(us, 'programs', 'operator.cap'), opCap);
  const logPath = join(base, logs);
  const fd = fsn.openSync(logPath, 'a');
  const proc = spawn(PINS.kernel.path, [], { env: { ...process.env,
    SKRYNIA_ROOT_WASM: PINS.p0.path, SKRYNIA_OPERATOR_DIR: op, SKRYNIA_USER_DIR: us,
    SKRYNIA_BIND: `127.0.0.1:${port}` }, stdio: ['ignore', fd, fd] });
  proc.on('close', () => { try { fsn.closeSync(fd); } catch {} });
  await waitTcp(port, proc, logPath);
  return { proc, op, us, logPath };
}

export async function startGateway({ dir, port, kernelPort, token, edgeCap, logs, appHost, appOrigin }) {
  await mkdir(dir, { recursive: true });
  const tokenFile = join(dir, 'operator.token');
  await writeFile(tokenFile, token, { mode: 0o600 });
  const host = `127.0.0.1:${port}`;
  const origin = `http://${host}`;
  const hosts = [...new Set([host, ...(appHost ? [appHost] : [])])];
  const origins = [...new Set([origin, ...(appOrigin ? [appOrigin] : [])])];
  const cfgPath = join(dir, 'config.json');
  await writeFile(cfgPath, JSON.stringify({
    bind: host,
    upstream: `http://127.0.0.1:${kernelPort}/`,
    operator_token_file: tokenFile,
    max_body_bytes: 4 * 1024 * 1024,
    apps: [{ name: 'webadmin-v3-e2e', hosts, origins, edge_capabilities: [edgeCap] }],
  }), { mode: 0o600 });
  const logPath = join(dir, logs);
  const fd = fsn.openSync(logPath, 'a');
  const proc = spawn(PINS.gateway.path, [], { env: { ...process.env, SKRYNIA_EDGE_GATEWAY_CONFIG: cfgPath },
    stdio: ['ignore', fd, fd] });
  proc.on('close', () => { try { fsn.closeSync(fd); } catch {} });
  await waitTcp(port, proc, logPath);
  return { proc, host, origin, edgeCap, logPath };
}

// Minimal injected fetch for the shipped adapter: adds the gateway-only
// session headers (edge cap + origin) that never live in the public bundle.
export function gatewayFetch({ host, origin, edgeCap }) {
  return (url, init) => new Promise((resolve, reject) => {
    const u = new URL(url);
    const body = Buffer.from(init.body);
    const req = http.request({ hostname: u.hostname, port: u.port, path: u.pathname + u.search,
      method: init.method || 'POST', headers: {
        Host: host, Origin: origin, 'X-Skrynia-Edge-Capability': edgeCap,
        'Content-Type': 'application/octet-stream', 'Content-Length': body.length,
      } }, res => {
      const chunks = []; res.on('data', c => chunks.push(c));
      res.on('end', () => {
        const buf = Buffer.concat(chunks);
        resolve({ status: res.statusCode, arrayBuffer: async () => buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength) });
      });
    });
    req.on('error', reject); req.setTimeout(20000, () => req.destroy(new Error('gateway timeout')));
    req.end(body);
  });
}

// Raw gateway request for auth-boundary negatives (bypasses the adapter).
export function rawGatewayPost({ host, origin, edgeCap, port, body }) {
  return new Promise((resolve, reject) => {
    const headers = { 'Content-Type': 'application/octet-stream', 'Content-Length': body.length };
    if (origin !== null) headers.Origin = origin;
    if (edgeCap !== null) headers['X-Skrynia-Edge-Capability'] = edgeCap;
    if (host !== null) headers.Host = host;
    const req = http.request({ hostname: '127.0.0.1', port, path: '/_skrynia/', method: 'POST', headers }, res => {
      res.resume(); res.on('end', () => resolve(res.statusCode));
    });
    req.on('error', reject); req.setTimeout(10000, () => req.destroy(new Error('gateway timeout')));
    req.end(Buffer.from(body));
  });
}

async function main() {
  for (const [name, a] of Object.entries(PINS)) {
    const okName = `pin ${name} sha256 ${a.sha256.slice(0, 12)}`;
    rec(okName, (await exists(a.path)) && (await hashFile(a.path)) === a.sha256, a.note);
    assert.equal(await hashFile(a.path), a.sha256, `pinned ${name} sha256 mismatch`);
  }

  const tmp = await mkdtemp(join(tmpdir(), 'webadmin-v3-native-e2e-'));
  const procs = [];
  const token = randomBytes(32);
  const tokenHex = Buffer.from(token).toString('hex');
  const verifierHex = Buffer.from(sha256buf(token)).toString('hex');
  const opCap = sha256buf(Buffer.from('webadmin-v3-e2e-operator-cap'));
  const adminCap = sha256buf(Buffer.from('webadmin-v3-e2e-admin-cap'));
  const iid = sha256buf(adminCap);
  let ok = false;
  try {
    const kernelPort = await freePort();
    const kernel = await startKernel({ base: join(tmp, 'kernel'), port: kernelPort, token, opCap, logs: 'kernel.log' });
    procs.push(kernel.proc);
    rec('pinned kernel booted with P0 root + ordinary P1 service', kernel.proc.exitCode === null,
      `bind 127.0.0.1:${kernelPort}`);

    const edgePort = await freePort();
    const edgeCap = 'webadmin-v3-e2e-' + randomBytes(16).toString('hex');
    const gw = await startGateway({ dir: join(tmp, 'edge-gw'), port: edgePort, kernelPort, token, edgeCap, logs: 'gateway.log' });
    procs.push(gw.proc);
    rec('trusted edge gateway booted with server-only operator token', gw.proc.exitCode === null,
      `bind ${gw.host}`);

    // ---------- A) real end-to-end through the shipped web-admin adapter ----------
    const transport = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
    rec('adapter bound to trusted gateway endpoint', transport.endpoint === `http://${gw.host}/_skrynia/`);
    const op = async (payload, label) => {
      const resp = await transport.post(payload);
      assert.ok(resp instanceof Uint8Array, `${label}: adapter must return bytes`);
      const d = decodeResponse(resp);
      rec(label, d.status === 0, `native status ${d.status}, body ${d.body.length}B`);
      return d;
    };
    const echoWasm = await readFile(PINS.echo.path);
    await op(registerOp(opCap, adminCap, echoWasm), 'A1 register ordinary private guest via adapter');
    await op(lookupOp(iid), 'A2 lookup registered guest via adapter');
    const invoked = await op(invokeOp(iid, Buffer.from('webadmin-v3-e2e-payload')), 'A3 invoke ordinary guest via adapter');
    rec('A3b ordinary service echoed payload byte-exact', invoked.body.toString() === 'webadmin-v3-e2e-payload');

    const key = Buffer.from('app/release-pointer');
    const v1 = Buffer.from('release-A');
    await op(appPutOp(adminCap, key, v1), 'A4 AppPut initial value via adapter');
    const got1 = await op(appGetOp(iid, key), 'A5 AppGet initial value via adapter');
    rec('A5b value round-tripped through P0 to ordinary service', got1.body.toString() === 'release-A');

    const v2 = Buffer.from('release-B');
    await op(appCasOp(adminCap, key, sha256buf(v1), v2), 'A6 bounded CAS via adapter');
    const got2 = await op(appGetOp(iid, key), 'A7 AppGet after CAS via adapter');
    rec('A7b CAS updated value through the real stack', got2.body.toString() === 'release-B');
    const gotAgain = await op(appGetOp(iid, key), 'A8 repeatable read (rollback target readable)');
    rec('A8b repeat read stable', gotAgain.body.toString() === 'release-B');

    // ---------- B) forged P0 framing denied by the REAL pinned P0 ----------
    const payload = lookupOp(iid);
    const badFrames = [
      ['B1 wrong version 0x00 denied', Buffer.concat([Buffer.from([0x00]), token, payload])],
      ['B2 wrong version 0x02 denied', Buffer.concat([Buffer.from([0x02]), token, payload])],
      ['B3 wrong version 0xff denied', Buffer.concat([Buffer.from([0xff]), token, payload])],
      ['B4 wrong token denied', framed(Buffer.from(Uint8Array.from(token, (b, i) => i ? b : b ^ 1)), payload)],
      ['B5 zero token denied', framed(new Uint8Array(32), payload)],
      ['B6 digest-as-token denied', framed(sha256buf(token), payload)],
      ['B7 empty frame denied', Buffer.alloc(0)],
      ['B8 1-byte frame denied', Buffer.from([0x01])],
      ['B9 31-byte truncated header denied', Buffer.concat([Buffer.from([0x01]), token.subarray(0, 30)])],
      ['B10 32-byte truncated header denied', Buffer.concat([Buffer.from([0x01]), token.subarray(0, 31)])],
    ];
    for (const [name, body] of badFrames) {
      const r = await rawKernelBody(kernelPort, body);
      let denied = r.http >= 400;
      if (!denied) { try { denied = decodeResponse(r.body).status !== 0; } catch { denied = true; } }
      const hex = r.body.toString('hex');
      const leaked = hex.includes(tokenHex) || hex.includes(verifierHex);
      rec(name, denied && !leaked, `http ${r.http}${leaked ? ' SECRET LEAK' : ''}`);
    }
    // Liveness: P0 denied frames must not kill the kernel or corrupt state.
    const alive = await op(appGetOp(iid, key), 'B11 kernel alive after forged frames');
    rec('B11b state unchanged after denial', alive.body.toString() === 'release-B');

    // ---------- C) gateway auth boundary fail-closed ----------
    const benign = Buffer.from([0x01, 0x04]);
    const gwNeg = [
      ['C1 wrong edge capability 401', { host: gw.host, origin: gw.origin, edgeCap: 'wrong', port: edgePort }, 401],
      ['C2 missing edge capability 401', { host: gw.host, origin: gw.origin, edgeCap: null, port: edgePort }, 401],
      ['C3 wrong origin 403', { host: gw.host, origin: 'http://evil.example', edgeCap, port: edgePort }, 403],
      ['C4 wrong host 403', { host: 'evil.example', origin: gw.origin, edgeCap, port: edgePort }, 403],
    ];
    for (const [name, opts, expected] of gwNeg) {
      const code = await rawGatewayPost({ ...opts, body: benign });
      rec(name, code === expected, `http ${code}`);
    }

    const evidence = {
      kind: 'web-admin-v3 real native HTTP E2E',
      p0_frame_pin: P0_FRAME_PIN,
      verified_at: new Date().toISOString(),
      pins: Object.fromEntries(Object.entries(PINS).map(([k, v]) => [k, { path: v.path, sha256: v.sha256, note: v.note }])),
      adapter: ADAPTER,
      adapter_endpoint: `http://${gw.host}/_skrynia/`,
      processes: { kernel: PINS.kernel.path, gateway: PINS.gateway.path },
      token_policy: 'random 32-byte bearer per run, 0600 temp file, never logged; no root/issuer token in public assets/URLs/logs',
      results,
      ALL_OK: results.every(r => r.ok),
    };
    if (process.env.E2E_EVIDENCE_OUT) {
      await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n');
    }
    console.log(JSON.stringify(evidence, null, 2));
    console.log(evidence.ALL_OK
      ? 'PASS web-admin-v3 real native HTTP E2E: adapter+trusted gateway+kernel+P0+ordinary service end-to-end, forged P0 framing fail-closed'
      : 'FAIL web-admin-v3 real native HTTP E2E');
    ok = evidence.ALL_OK;
    if (!ok) process.exitCode = 1;
  } finally {
    for (const p of procs) { try { await stopProc(p); } catch {} }
    await delay(200);
    if (ok) await rm(tmp, { recursive: true, force: true });
    else console.error('preserved tmp for diagnosis:', tmp);
  }
}

// Reusable as a library (see chromium.mjs); run only when invoked directly.
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
}
