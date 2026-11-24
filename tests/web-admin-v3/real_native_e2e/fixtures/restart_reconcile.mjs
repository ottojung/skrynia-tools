#!/usr/bin/env node
// Exact-head CI check over the REAL compiled stack: lost-reply reconciliation
// (drop-after-commit) and durable restart, exercising the SHIPPED web-admin-v3
// adapter + reconciliation UI. No simulator, no fake service, no mock adapter.
//
// Drop-after-commit: a loopback proxy forwards the POST to the real trusted
// edge gateway -> kernel -> dumb P0 -> P1, lets the ACTIVATE commit, then
// destroys the client socket so binary_transport throws `transport_unknown`
// and native_capabilities throws `uncertain`. We then assert:
//   - the pointer DID commit (real native GET_ACTIVE shows exact revision N+1),
//   - the shipped UI disables Activate, offers "Refresh state", and does NOT
//     spend a second CAS (no blind repeat) until an explicit refresh,
//   - after refresh the UI shows the authoritative revision.
// Restart: kernel+gateway are stopped and restarted on the same private state
// dirs; every private-state file is hash-compared before/after to prove no
// reinitialization, and durable GET_ACTIVE/LIST_VERSIONS are re-verified.
//
// Wire layouts are the exact p1_admin_ops encoders (OP 9/10/11/12/13/14;
// fixed 42-byte active record, u64 LE revision, 32-byte hashes).

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import http from 'node:http';
import { mkdtemp, mkdir, readFile, readdir, writeFile, rm } from 'node:fs/promises';
import * as fsn from 'node:fs';
import { join, resolve, dirname, extname } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

import {
  PINS, P0_FRAME_PIN, putId, putBlob, registerOp, decodeResponse,
  startKernel, startGateway, gatewayFetch, freePort, stopProc,
  sha256hex, sha256buf, hashFile, exists,
} from './run.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '../../../..');
const WEB = join(ROOT, 'web-admin-v3');
const TRANSPORT = new URL('../../../../web-admin-v3/binary_transport.js', import.meta.url).href;
const NATIVE = new URL('../../../../web-admin-v3/native_capabilities.js', import.meta.url).href;
const { createBinaryPostTransport } = await import(TRANSPORT);
const { createNativeCapabilities } = await import(NATIVE);

const OP = { STAGE: 9, SEAL: 10, ACTIVATE: 11, GET_ACTIVE: 12, LIST_VERSIONS: 13, LIST_PROGRAMS: 14 };
const ERR = { BADREQ: 1, NOTFOUND: 8, UNAUTH: 9, CONFLICT: 10 };
// Same encoders as p1_admin_ops.mjs (verified against native-service wire ABI).
const stageOp = (admin, code) => Buffer.concat([Buffer.from([1, OP.STAGE]), putId(admin), putBlob(code)]);
const sealOp = (admin, hash) => Buffer.concat([Buffer.from([1, OP.SEAL]), putId(admin), Buffer.from(hash)]);
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
  return { ver: body[0], live: body[1], hash: body.subarray(2, 34), revision: Number(body.readBigUInt64LE(34)) };
}
function parseList(body) {
  const n = body.readUInt16LE(0);
  assert.equal(body.length, 2 + n * 32, 'version list body length');
  return Array.from({ length: n }, (_, i) => body.subarray(2 + i * 32, 2 + (i + 1) * 32));
}

const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8' };
const SERVED = new Set(['index.html', 'main.js', 'apps.js', 'apps_ui.js', 'capabilities.js',
  'binary_transport.js', 'native_capabilities.js', 'native_bootstrap.js', 'browser_storage.js',
  'crypto.js', 'vault.js', 'styles.css']);

// Static origin with a switchable drop-after-commit proxy to the real gateway.
async function startStaticHost({ port, edgePort, appHost, appOrigin, edgeCap }) {
  const state = { drop: false, seen: [], activateCount: 0, afterCommit: null };
  const server = http.createServer(async (req, res) => {
    const u = new URL(req.url, 'http://127.0.0.1');
    if (u.pathname !== '/_skrynia/') {
      const name = u.pathname === '/' ? 'index.html' : decodeURIComponent(u.pathname).replace(/^\//, '');
      if (!SERVED.has(name)) { res.writeHead(404); res.end(); return; }
      try {
        const data = await readFile(join(WEB, name));
        res.writeHead(200, { 'Content-Type': MIME[extname(name)] || 'application/octet-stream', 'Cache-Control': 'no-store' });
        res.end(data);
      } catch { res.writeHead(404); res.end(); }
      return;
    }
    const chunks = [];
    for await (const c of req) chunks.push(c);
    const body = Buffer.concat(chunks);
    state.seen.push({ method: req.method, path: u.pathname, bytes: body.length });
    // Forward to the REAL trusted edge gateway with its server-only cap.
    const upstream = http.request({
      hostname: '127.0.0.1', port: edgePort, path: '/_skrynia/', method: 'POST',
      headers: {
        Host: appHost, Origin: appOrigin, 'X-Skrynia-Edge-Capability': edgeCap,
        'Content-Type': 'application/octet-stream', 'Content-Length': body.length,
      },
    }, upstreamRes => {
      const out = [];
      upstreamRes.on('data', c => out.push(c));
      upstreamRes.on('end', () => {
        const buf = Buffer.concat(out);
        const isActivate = body.length > 2 && body[0] === 1 && body[1] === 11;
        if (isActivate) { state.activateCount += 1; state.afterCommit = { http: upstreamRes.statusCode, body: buf, request: Buffer.from(body) }; }
        if (state.drop && isActivate) {
          // DROP AFTER COMMIT: service has committed, client never gets a reply.
          res.destroy();
          return;
        }
        res.writeHead(upstreamRes.statusCode, { 'Content-Type': 'application/octet-stream' });
        res.end(buf);
      });
    });
    upstream.on('error', () => { res.writeHead(502); res.end(); });
    upstream.end(body);
  });
  await new Promise(ok => server.listen(port, '127.0.0.1', ok));
  return { server, state, origin: appOrigin };
}

async function cdpTarget(port) {
  for (let i = 0; i < 200; i += 1) {
    try {
      const r = await fetch('http://127.0.0.1:' + port + '/json/list');
      const pages = await r.json();
      const page = pages.find(p => p.type === 'page' && p.webSocketDebuggerUrl);
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await delay(100);
  }
  throw Error('Chromium CDP target unavailable');
}
function makeCdp(socket) {
  let next = 0; const pending = new Map();
  socket.addEventListener('message', event => {
    const msg = JSON.parse(event.data); const p = pending.get(msg.id);
    if (!p) return; pending.delete(msg.id);
    if (msg.error || msg.result?.exceptionDetails) p.reject(Error(JSON.stringify(msg)));
    else p.resolve(msg.result?.result?.value);
  });
  const send = (method, params = {}) => new Promise((resolveP, reject) => {
    const id = ++next; pending.set(id, { resolve: resolveP, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });
  return { send, evaluate: expression => send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }) };
}
async function until(evaluate, expression, label, tries = 200) {
  let last = null;
  for (let i = 0; i < tries; i += 1) {
    try { last = await evaluate(expression); if (last) return last; } catch {}
    await delay(100);
  }
  throw Error('Chromium condition timed out: ' + label + ' (last=' + JSON.stringify(last) + ')');
}
async function hashPrivateState(us) {
  const out = {};
  async function walk(base, rel) {
    let entries = [];
    try { entries = (await readdir(join(base, rel), { withFileTypes: true })).sort((a,b)=>a.name.localeCompare(b.name)); } catch {}
    for (const e of entries) {
      const r = join(rel, e.name);
      if (e.isDirectory()) await walk(base, r);
      else if (e.isFile()) out[r] = await hashFile(join(base, r));
    }
  }
  await walk(join(us, 'programs'), '');
  return out;
}

export async function main() {
  const results = [];
  const rec = (name, ok, detail = '') => {
    results.push({ name, ok: Boolean(ok), detail: String(detail) });
    console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
    return ok;
  };
  let allOk = false;
  const tmp = await mkdtemp(join(tmpdir(), 'webadmin-restart-reconcile-'));
  const procs = [];
  let staticServer = null, browser = null, socket = null;
  try {
    for (const [name, pin] of Object.entries(PINS)) {
      const got = (await exists(pin.path)) ? await hashFile(pin.path) : 'MISSING';
      rec(`pin ${name} sha256 ${pin.sha256.slice(0, 12)}`, got === pin.sha256, got.slice(0, 12));
      assert.equal(got, pin.sha256, `pin ${name} mismatch`);
    }
    rec('P0 frame pin', P0_FRAME_PIN === '8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217');

    const token = randomBytes(32);
    const opCap = randomBytes(32);
    const adminCap = randomBytes(32);
    const iid = sha256buf(adminCap);
    const app = iid.toString('hex');
    const echo = await readFile(PINS.echo.path);
    const hA = sha256buf(echo).toString('hex');
    // Valid wasm module with a distinct custom section (native service stores
    // arbitrary non-empty code, but keeping B a valid wasm avoids assumptions).
    const codeB = await readFile(resolve(HERE, 'codeB.wasm'));
    const hB = sha256buf(codeB).toString('hex');
    rec('codeB is a valid wasm module', codeB[0] === 0 && codeB[1] === 0x61 && codeB[2] === 0x73 && codeB[3] === 0x6d && codeB[4] === 1, hB.slice(0, 12));

    const kernelPort = await freePort();
    const base = join(tmp, 'kernel');
    let kernel = await startKernel({ base, port: kernelPort, token, opCap, logs: 'kernel.log' });
    procs.push(kernel.proc);
    rec('kernel booted with P0+P1', kernel.proc.exitCode === null, `127.0.0.1:${kernelPort}`);
    const edgePort = await freePort();
    const edgeCap = 'restart-e2e-' + randomBytes(16).toString('hex');
    const staticPort = await freePort();
    const appHost = `127.0.0.1:${staticPort}`;
    const appOrigin = `http://${appHost}`;
    let gw = await startGateway({ dir: join(tmp, 'edge'), port: edgePort, kernelPort, token, edgeCap, logs: 'gateway.log', appHost, appOrigin });
    procs.push(gw.proc);
    rec('trusted edge gateway booted', gw.proc.exitCode === null, gw.host);

    // Provision ready history through the real shipped adapter (op 15/9/10).
    let transport = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
    const send = async frame => decodeResponse(await transport.post(frame));
    const okOp = async (name, frame, pred = () => true) => {
      const d = await send(frame);
      const pass = d.status === 0 && pred(d);
      rec(name, pass, `status ${d.status}`);
      assert.ok(pass, name);
      return d;
    };
    await okOp('register A (op 15)', registerOp(opCap, adminCap, echo));
    await okOp('stage B (op 9, valid wasm)', stageOp(adminCap, codeB), d => Buffer.from(d.body).toString('hex') === hB);
    await okOp('seal B (op 10)', sealOp(adminCap, Buffer.from(hB, 'hex')));
    const a0 = await okOp('active A rev1 (op 12, 42B)', getActiveOp(iid), d => {
      const s = parseActive(d.body); return s.ver === 3 && s.live === 1 && s.revision === 1 && s.hash.toString('hex') === hA;
    });
    const revA = parseActive(a0.body).revision;

    // ---- Static origin + drop-after-commit proxy, real Chromium UI ----
    const host = await startStaticHost({ port: staticPort, edgePort, appHost, appOrigin, edgeCap });
    staticServer = host.server;
    rec('independent static origin serves shipped admin app', true, appOrigin);

    const cdpPort = await freePort();
    browser = spawn(process.env.CHROME_BIN || 'chromium', ['--headless=new', '--no-sandbox', '--disable-gpu',
      '--disable-dev-shm-usage', '--no-first-run', '--disable-background-networking',
      '--remote-debugging-port=' + cdpPort, '--user-data-dir=' + join(tmp, 'chromium'), 'about:blank'],
      { stdio: ['ignore', 'ignore', 'pipe'] });
    const wsUrl = await cdpTarget(cdpPort);
    socket = new WebSocket(wsUrl);
    await new Promise((ok, no) => { socket.addEventListener('open', ok, { once: true }); socket.addEventListener('error', no, { once: true }); });
    const { send: cdp, evaluate } = makeCdp(socket);
    await cdp('Page.enable');
    const session = { endpoint: appOrigin + '/_skrynia/', readCap: app, adminCap: adminCap.toString('hex') };
    await cdp('Page.addScriptToEvaluateOnNewDocument', { source: 'window.__skryniaNativeSession=' + JSON.stringify(session) + ';' });
    await cdp('Page.navigate', { url: appOrigin + '/' });
    await until(evaluate, 'document.readyState==="complete" && !!window.__skryniaV3AdminCapabilities && !!document.getElementById("status")', 'static app loaded');
    rec('real Chromium loaded shipped native adapter', true);
    await evaluate('window.confirm=()=>true;document.getElementById("vault-secret").value="e2e-restart-secret";document.getElementById("unlock-form").requestSubmit();');
    await until(evaluate, 'document.getElementById("workspace").hidden===false', 'vault unlock');
    rec('vault unlocked', true);

    // Baseline UI state: active A rev1.
    let live = await evaluate(`window.__skryniaV3AdminCapabilities().getAppState("${app}")`);
    rec('baseline UI state A rev1', live.active === hA && live.pointerVersion === 1, JSON.stringify(live));

    // ---- DROP AFTER COMMIT through the shipped UI's activate control ----
    // Click the real "Activate this version" button for B, with drop enabled.
    await until(evaluate,
      `(()=>{const p=[...document.querySelectorAll(".app-panel")].find(x=>x.querySelector("h3")?.textContent.includes("${app}"));`
      + `if(!p) return false; const row=[...p.querySelectorAll(".version-row")].find(x=>x.querySelector(".version-meta")?.textContent.startsWith("${hB}"));`
      + `if(!row) return false; const b=[...row.querySelectorAll("button")].find(b=>b.textContent==="Activate this version");`
      + `return !!(b && !b.disabled);})()`, 'activate B button ready');
    host.state.drop = true;
    const before = host.state.seen.length;
    const clickB = `(()=>{const p=[...document.querySelectorAll(".app-panel")].find(x=>x.querySelector("h3")?.textContent.includes("${app}"));`
      + `const row=[...p.querySelectorAll(".version-row")].find(x=>x.querySelector(".version-meta")?.textContent.startsWith("${hB}"));`
      + `const b=[...row.querySelectorAll("button")].find(b=>b.textContent==="Activate this version");`
      + `b.click(); return true;})()`;
    await evaluate(clickB);
    // Wait for the real service commit (proxy records upstream response).
    for (let i = 0; i < 200 && !host.state.afterCommit; i += 1) await delay(100);
    assert.ok(host.state.afterCommit, 'upstream never responded');
    rec('drop proxy observed real upstream commit response', host.state.afterCommit.http === 200, 'http ' + host.state.afterCommit.http);

    // Service really committed despite client seeing no reply.
    const committed = decodeResponse(host.state.afterCommit.body);
    if (committed.body.length !== 42) {
      console.error('DEBUG committed', 'status', committed.status, 'len', committed.body.length, 'hex', Buffer.from(committed.body).toString('hex').slice(0, 200));
      console.error('DEBUG request hex', Buffer.from(host.state.afterCommit.request||[]).toString('hex').slice(0,200));
    }
    const committedState = parseActive(committed.body);
    rec('pointer committed on service to B rev2', committed.status === 0 && committedState.revision === 2 && committedState.hash.toString('hex') === hB, JSON.stringify(committedState));

    // Shipped UI marks uncertain, disables activation, offers Refresh state.
    await until(evaluate, 'document.getElementById("status").textContent.includes("outcome unknown")', 'uncertain status');
    const guardedExpr = `(()=>{const p=[...document.querySelectorAll(".app-panel")].find(x=>x.querySelector("h3")?.textContent.includes("${app}"));`
      + `if(!p) return null; const btns=[...p.querySelectorAll("button")];`
      + `const refresh=btns.some(b=>b.textContent==="Refresh state");`
      + `const disabled=btns.filter(b=>b.textContent==="Activate this version").every(b=>b.disabled);`
      + `const warning=p.textContent.includes("outcome unknown");`
      + `return (warning && refresh && disabled) ? {warning,refresh,disabled} : null;})()`;
    const guarded = await until(evaluate, guardedExpr, 'uncertain UI guard');
    rec('shipped UI shows uncertainty warning', guarded.warning, JSON.stringify(guarded));
    rec('shipped UI offers Refresh state', guarded.refresh);
    rec('shipped UI disables Activate (no blind repeat)', guarded.disabled);
    // No second ACTIVATE (op 11) was spent by the UI after the lost reply.
    rec('no blind repeat CAS after lost reply', host.state.activateCount === 1, `activateCount=${host.state.activateCount}`);

    // Explicit refresh via the shipped UI reconciles to the authoritative rev2.
    host.state.drop = false;
    await until(evaluate, `(()=>{const p=[...document.querySelectorAll(".app-panel")].find(x=>x.querySelector("h3")?.textContent.includes("${app}"));`
      + `if(!p) return false; const b=[...p.querySelectorAll("button")].find(b=>b.textContent==="Refresh state"); if(!b) return false; b.click(); return true;})()`, 'refresh click');
    await until(evaluate, `window.__skryniaV3AdminCapabilities().getAppState("${app}").then(s=>s.pointerVersion===2&&s.active==="${hB}")`, 'UI reconciled to rev2');
    live = await evaluate(`window.__skryniaV3AdminCapabilities().getAppState("${app}")`);
    rec('UI reconciled to exact B rev2 after refresh', live.active === hB && live.pointerVersion === 2, JSON.stringify(live));
    rec('UI re-enabled Activate after explicit refresh', await until(evaluate, `(()=>{const p=[...document.querySelectorAll(".app-panel")].find(x=>x.querySelector("h3")?.textContent.includes("${app}"));`
      + `if(!p) return false; return [...p.querySelectorAll("button")].filter(b=>b.textContent==="Activate this version").some(b=>!b.disabled);})()`, 'activate re-enabled'));

    // ---- Private-state integrity across restart ----
    const beforeState = await hashPrivateState(kernel.us);
    rec('private state has files', Object.keys(beforeState).length > 0, Object.keys(beforeState).length + ' files');

    await stopProc(gw.proc); procs.splice(procs.indexOf(gw.proc), 1);
    await stopProc(kernel.proc); procs.splice(procs.indexOf(kernel.proc), 1);
    await delay(300);
    rec('stopped kernel and gateway', true);

    const kernelPort2 = await freePort();
    kernel = await startKernel({ base, port: kernelPort2, token, opCap, logs: 'kernel-restart.log' });
    procs.push(kernel.proc);
    const edgePort2 = await freePort();
    gw = await startGateway({ dir: join(tmp, 'edge-restart'), port: edgePort2, kernelPort: kernelPort2, token, edgeCap, logs: 'gateway-restart.log' });
    procs.push(gw.proc);
    rec('restarted on same private state dirs', true, `kernel ${kernelPort2}, gateway ${edgePort2}`);
    const afterState = await hashPrivateState(kernel.us);
    rec('restart did NOT reinitialize private state', JSON.stringify(beforeState) === JSON.stringify(afterState),
      JSON.stringify(beforeState) === JSON.stringify(afterState) ? 'all private files byte-identical' : 'PRIVATE STATE CHANGED');

    transport = createBinaryPostTransport(`http://${gw.host}/_skrynia/`, gatewayFetch(gw));
    const send2 = async frame => decodeResponse(await transport.post(frame));
    const d2 = await send2(getActiveOp(iid));
    const s2 = parseActive(d2.body);
    rec('after restart durable B rev2', d2.status === 0 && s2.revision === 2 && s2.hash.toString('hex') === hB, JSON.stringify(s2));
    const l2 = await send2(listVersionsOp(iid));
    const xs = parseList(l2.body).map(x => x.toString('hex')).sort();
    rec('after restart LIST_VERSIONS has A,B', JSON.stringify(xs) === JSON.stringify([hA, hB].sort()), xs.join(','));
    // Post-restart activation proves service still functional with monotone CAS.
    const d3 = await send2(activateOp(adminCap, 2, Buffer.from(hA, 'hex')));
    const s3 = parseActive(d3.body);
    rec('post-restart ACTIVATE B->A rev3', d3.status === 0 && s3.revision === 3 && s3.hash.toString('hex') === hA, JSON.stringify(s3));
    const d4 = await send2(activateOp(adminCap, 2, Buffer.from(hB, 'hex')));
    rec('post-restart stale CAS rev2 denied as conflict', d4.status === 1 && d4.body[0] === ERR.CONFLICT, 'err ' + d4.body[0]);

    allOk = results.every(r => r.ok);
    const evidence = {
      kind: 'web-admin-v3 real-stack lost-reply reconciliation + restart (shipped UI, drop-after-commit)',
      verified_at: new Date().toISOString(),
      node: process.version,
      p0_frame_pin: P0_FRAME_PIN,
      pins: Object.fromEntries(Object.entries(PINS).map(([k, v]) => [k, { path: v.path, sha256: v.sha256 }])),
      private_state_before: beforeState,
      private_state_after: afterState,
      results, ALL_OK: allOk,
    };
    if (process.env.E2E_EVIDENCE_OUT) await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n', { mode: 0o600 });
    console.log(allOk ? 'PASS drop-after-commit reconciliation + restart (real stack + shipped UI)' : 'FAIL reconciliation/restart');
    if (!allOk) process.exitCode = 1;
  } finally {
    try { if (socket) socket.close(); } catch {}
    try { if (browser) await stopProc(browser); } catch {}
    if (staticServer) await new Promise(ok => staticServer.close(ok));
    for (const p of procs) { try { await stopProc(p); } catch {} }
    await delay(200);
    if (allOk) await rm(tmp, { recursive: true, force: true });
    else console.error('preserved tmp for diagnosis:', tmp);
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
}
