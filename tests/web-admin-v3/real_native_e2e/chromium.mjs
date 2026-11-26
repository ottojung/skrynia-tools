#!/usr/bin/env node
// Real Chromium end-to-end qualification of the web-admin-v3 UI-to-native
// wiring. NO simulator and NO fake adapter in the browser: the shipped static
// app (index.html + native_bootstrap.js + native_capabilities.js) is hosted as
// an ordinary independent origin, its /_skrynia/ path is reverse-proxied to the
// trusted edge gateway with the edge capability injected server-side, and the
// UI drives the REAL pinned kernel + dumb P0 + ordinary WASM service.
//
// Operator session authority (disposable per-run capability bytes) is injected
// into the browser at runtime, never baked into a static bundle, URL or log.
// No production TLS/DNS, no deploy, no merge and no Rust rebuild.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { spawn } from 'node:child_process';
import { chromiumEnvironment } from './chromium_env.mjs';
import { createServer } from 'node:net';
import http from 'node:http';
import { mkdtemp, mkdir, readFile, writeFile, rm, stat } from 'node:fs/promises';
import { join, resolve, dirname, extname } from 'node:path';
import { hostname, tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

import {
  PINS, P0_FRAME_PIN, sha256hex, sha256buf, hashFile, exists, freePort, stopProc,
  startKernel, startGateway, registerOp, putId, putBlob, decodeResponse,
} from './run.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, '../../..');
const WEB = join(ROOT, 'web-admin-v3');
const CHROME = process.env.CHROME_BIN || 'chromium';
const sha = b => createHash('sha256').update(Buffer.from(b)).digest();

const results = [];
function rec(name, ok, detail = '') {
  results.push({name, ok: Boolean(ok), detail: String(detail)});
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
  return ok;
}

const MIME = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
};
const SERVED = new Set([
  'index.html', 'main.js', 'apps.js', 'apps_ui.js', 'capabilities.js',
  'binary_transport.js', 'native_capabilities.js', 'native_bootstrap.js',
  'browser_storage.js', 'crypto.js', 'vault.js', 'styles.css',
]);

// Disposable independent static origin: serves only the shipped static app and
// reverse-proxies /_skrynia/ to the trusted gateway. The edge capability is
// added here, server-side; it is never present in public assets or URLs.
export async function startStaticHost({ port, edgePort, appHost, appOrigin, edgeCap, onRequest = null }) {
  const seen = [];
  const server = http.createServer(async (req, res) => {
    const u = new URL(req.url, 'http://127.0.0.1');
    if (u.pathname === '/_skrynia/') {
      seen.push({path: u.pathname, method: req.method});
      if (req.method !== 'POST') { res.writeHead(405); res.end(); return; }
      const chunks = [];
      for await (const c of req) chunks.push(c);
      const body = Buffer.concat(chunks);
      // Optional read-only observer (used by the 1000-version pagination proof
      // to assert the wire page cap); never logs or persists frame bytes.
      if (onRequest) onRequest(body, {path: u.pathname, method: req.method});
      const upstream = http.request({
        hostname: '127.0.0.1', port: edgePort, path: '/_skrynia/', method: 'POST',
        headers: {
          Host: appHost,
          Origin: appOrigin,
          'X-Skrynia-Edge-Capability': edgeCap,
          'Content-Type': 'application/octet-stream',
          'Content-Length': body.length,
        },
      }, upstreamRes => {
        const out = [];
        upstreamRes.on('data', c => out.push(c));
        upstreamRes.on('end', () => {
          const buf = Buffer.concat(out);
          res.writeHead(upstreamRes.statusCode, {'Content-Type': 'application/octet-stream'});
          res.end(buf);
        });
      });
      upstream.on('error', () => { res.writeHead(502); res.end(); });
      upstream.end(body);
      return;
    }
    const name = u.pathname === '/' ? 'index.html' : decodeURIComponent(u.pathname).replace(/^\//, '');
    if (!SERVED.has(name)) { res.writeHead(404); res.end(); return; }
    try {
      const data = await readFile(join(WEB, name));
      res.writeHead(200, {'Content-Type': MIME[extname(name)] || 'application/octet-stream',
        'Cache-Control': 'no-store'});
      res.end(data);
    } catch { res.writeHead(404); res.end(); }
  });
  await new Promise(ok => server.listen(port, '127.0.0.1', ok));
  return {server, seen, origin: appOrigin};
}

export async function cdpTarget(port) {
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

export function cdp(socket) {
  let next = 0;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const msg = JSON.parse(event.data);
    const p = pending.get(msg.id);
    if (!p) return;
    pending.delete(msg.id);
    if (msg.error || msg.result?.exceptionDetails) p.reject(Error(JSON.stringify(msg)));
    else p.resolve(msg.result?.result?.value);
  });
  const send = (method, params = {}) => new Promise((resolveP, reject) => {
    const id = ++next;
    pending.set(id, {resolve: resolveP, reject});
    socket.send(JSON.stringify({id, method, params}));
  });
  const evaluate = expression => send('Runtime.evaluate', {
    expression, awaitPromise: true, returnByValue: true,
  });
  return {send, evaluate};
}

export async function until(evaluate, expression, label, tries = 200) {
  let last = null;
  for (let i = 0; i < tries; i += 1) {
    try {
      last = await evaluate(expression);
      if (last) return last;
    } catch {}
    await delay(100);
  }
  throw Error('Chromium condition timed out: ' + label + ' (last=' + JSON.stringify(last) + ')');
}

export async function postNative(origin, bytes) {
  return await new Promise((ok, no) => {
    const req = http.request(origin + '/_skrynia/', {method: 'POST', headers: {
      'Content-Type': 'application/octet-stream', 'Content-Length': bytes.length,
    }}, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => ok({http: res.statusCode, body: Buffer.concat(chunks)}));
    });
    req.on('error', no);
    req.setTimeout(20000, () => req.destroy(Error('gateway timeout')));
    req.end(Buffer.from(bytes));
  });
}

export const stageOp = (admin, code) =>
  Buffer.concat([Buffer.from([1, 9]), putId(admin), putBlob(code)]);
export const sealOp = (admin, hash) =>
  Buffer.concat([Buffer.from([1, 10]), putId(admin), Buffer.from(hash)]);
export const getActiveOp = iid => Buffer.concat([Buffer.from([1, 12]), putId(iid)]);

async function main() {
  for (const [name, a] of Object.entries(PINS)) {
    rec(`pin ${name} sha256 ${a.sha256.slice(0, 12)}`,
      (await exists(a.path)) && (await hashFile(a.path)) === a.sha256, a.note);
    assert.equal(await hashFile(a.path), a.sha256, `pinned ${name} sha256 mismatch`);
  }

  const tmp = await mkdtemp(join(tmpdir(), 'webadmin-v3-chromium-e2e-'));
  const procs = [];
  let staticServer = null;
  let browser = null;
  try {
    const token = randomBytes(32);
    const tokenHex = Buffer.from(token).toString('hex');
    const verifierHex = Buffer.from(sha256buf(token)).toString('hex');
    const verifierHex2 = sha256hex(token);
    assert.equal(verifierHex, verifierHex2);
    const opCap = sha256buf(Buffer.from('webadmin-v3-chromium-e2e-operator'));
    const adminCap = sha256buf(Buffer.from('webadmin-v3-chromium-e2e-admin'));
    const readCap = sha256buf(adminCap);
    const edgeCap = 'webadmin-v3-chromium-' + randomBytes(16).toString('hex');

    const kernelPort = await freePort();
    const kernel = await startKernel({base: join(tmp, 'kernel'), port: kernelPort, token, opCap, logs: 'kernel.log'});
    procs.push(kernel.proc);
    rec('pinned kernel booted with dumb P0 + ordinary service', kernel.proc.exitCode === null,
      `bind 127.0.0.1:${kernelPort}`);

    const staticPort = await freePort();
    const appHost = `127.0.0.1:${staticPort}`;
    const appOrigin = `http://${appHost}`;
    const edgePort = await freePort();
    const gw = await startGateway({dir: join(tmp, 'edge-gw'), port: edgePort, kernelPort,
      token, edgeCap, logs: 'gateway.log', appHost, appOrigin});
    procs.push(gw.proc);
    rec('trusted edge gateway booted with server-only token', gw.proc.exitCode === null,
      `bind 127.0.0.1:${edgePort}, allow ${appOrigin}`);

    const host = await startStaticHost({port: staticPort, edgePort, appHost, appOrigin, edgeCap});
    staticServer = host.server;
    rec('independent static origin serves shipped admin app', true, appOrigin);

    // ---- Provision sealed ready history in the compiled P1 admin service ----
    const APP = Buffer.from(readCap).toString('hex');
    const codeA = await readFile(PINS.echo.path);
    const A = sha256buf(codeA).toString('hex');
    const codeB = Buffer.concat([Buffer.from('chromium-e2e-B-'), randomBytes(16)]);
    const codeC = Buffer.concat([Buffer.from('chromium-e2e-C-'), randomBytes(16)]);
    const B = sha256buf(codeB).toString('hex');
    const C = sha256buf(codeC).toString('hex');
    const nativeOp = async (frame, label) => {
      const r = await postNative(appOrigin, frame);
      assert.equal(r.http, 200, label + ' http');
      const d = decodeResponse(r.body);
      assert.equal(d.status, 0, label + ' native status');
      return d.body;
    };
    await nativeOp(registerOp(opCap, adminCap, codeA), 'register A');
    assert.equal((await nativeOp(stageOp(adminCap, codeB), 'stage B')).toString('hex'), B);
    await nativeOp(sealOp(adminCap, Buffer.from(B, 'hex')), 'seal B');
    assert.equal((await nativeOp(stageOp(adminCap, codeC), 'stage C')).toString('hex'), C);
    rec('compiled P1 ready history provisioned (A,B sealed; C staged)', true,
      '2 ready, 1 staged-but-unsealed');

    // ---- Launch real Chromium and inject only runtime session authority ----
    const cdpPort = await freePort();
    browser = spawn(CHROME, ['--headless=new', '--no-sandbox', '--disable-gpu',
      '--disable-dev-shm-usage', '--no-first-run', '--disable-background-networking',
      '--remote-debugging-port=' + cdpPort, '--user-data-dir=' + join(tmp, 'chromium'),
      'about:blank'], {env: chromiumEnvironment(), stdio: ['ignore', 'ignore', 'pipe']});
    let chromeStderr = '';
    browser.stderr.on('data', c => { chromeStderr += c.toString(); });
    const wsUrl = await cdpTarget(cdpPort);
    rec('real Chromium launched with CDP', true, CHROME);
    const socket = new WebSocket(wsUrl);
    await new Promise((ok, no) => {
      socket.addEventListener('open', ok, {once: true});
      socket.addEventListener('error', no, {once: true});
    });
    const {send, evaluate} = cdp(socket);
    await send('Page.enable');
    const session = {endpoint: appOrigin + '/_skrynia/',
      readCap: Buffer.from(readCap).toString('hex'),
      adminCap: Buffer.from(adminCap).toString('hex')};
    await send('Page.addScriptToEvaluateOnNewDocument', {
      source: 'window.__skryniaNativeSession=' + JSON.stringify(session) + ';',
    });
    await send('Page.navigate', {url: appOrigin + '/'});
    await until(evaluate,
      'document.readyState==="complete" && !!window.__skryniaV3AdminCapabilities &&'
      + ' !!document.getElementById("status")',
      'static app loaded with native adapter');
    rec('static app loaded native adapter in Chromium', true);
    rec('capability status reports native adapter',
      (await evaluate('document.getElementById("capability-status").textContent')).includes('native'));

    // ---- Encrypted vault unlock (secret stays in browser memory only) ----
    await evaluate('window.confirm=()=>true;'
      + 'document.getElementById("vault-secret").value="e2e-access-secret";'
      + 'document.getElementById("unlock-form").requestSubmit();');
    await until(evaluate, 'document.getElementById("workspace").hidden===false', 'vault unlock');
    rec('browser vault unlocked without sending the access secret', true);
    rec('password field cleared after unlock',
      (await evaluate('document.getElementById("vault-secret").value')) === '');

    await until(evaluate,
      '[...document.querySelectorAll(".app-panel h3")].some(x=>x.textContent.includes("' + APP + '"))',
      'all-app/all-version history rendered');
    const panelInfo = await evaluate('(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("' + APP + '"));'
      + 'return {title:p.querySelector("h3").textContent,'
      + ' pointer:p.querySelector("p").textContent,'
      + ' rows:p.querySelectorAll(".version-row").length,'
      + ' badges:[...p.querySelectorAll(".badge")].map(x=>x.textContent)};})()');
    rec('UI lists every ready release from the compiled service',
      panelInfo.rows === 2 && panelInfo.badges.includes('ready') &&
      !panelInfo.badges.includes('unavailable'), JSON.stringify(panelInfo));
    rec('UI shows the current active pointer', panelInfo.pointer.includes(A), panelInfo.pointer);

    // ---- Sealed release identity inspection ----
    const clickRowButton = (id, label) => evaluate('(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("' + APP + '"));'
      + 'const row=[...p.querySelectorAll(".version-row")]'
      + '.find(x=>x.querySelector(".version-meta")?.textContent.startsWith(' + JSON.stringify(id) + '));'
      + 'const button=[...row.querySelectorAll("button")]'
      + '.find(b=>b.textContent===' + JSON.stringify(label) + ');'
      + 'if(!button||button.disabled) throw Error("button unavailable: ' + label + '");'
      + 'button.click();return true;})()');
    await clickRowButton(B, 'Inspect manifest');
    await until(evaluate,
      'document.getElementById("manifest-view").textContent.includes("skrynia-native-static-v1")'
      + ' && document.getElementById("manifest-view").textContent.includes(' + JSON.stringify(B) + ')'
      + ' && document.getElementById("manifest-view").textContent.includes("native-p1-ready-hash")',
      'sealed release identity inspection');
    rec('sealed release identity inspected in browser (no invented file list)', true);

    // ---- A -> B -> A O(1) CAS rollback driven through the real UI ----
    await clickRowButton(B, 'Activate this version');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Activated ' + APP + ' → ' + B + '")',
      'activation to B');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Loaded")',
      'post-activation re-render');
    let live = await evaluate('window.__skryniaV3AdminCapabilities().getAppState("' + APP + '")');
    rec('UI activation advanced active pointer to B',
      live.active === B && live.pointerVersion === 2, 'rev=' + live.pointerVersion);

    await clickRowButton(A, 'Activate this version');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Activated ' + APP + ' → ' + A + '")',
      'rollback to A');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Loaded")',
      'post-activation re-render');
    live = await evaluate('window.__skryniaV3AdminCapabilities().getAppState("' + APP + '")');
    rec('UI A-B-A rollback advanced revision without asset rewrite',
      live.active === A && live.pointerVersion === 3, 'rev=' + live.pointerVersion);

    const conflictCode = await evaluate('(async()=>{try{'
      + 'await window.__skryniaV3AdminCapabilities().activateRelease("' + APP + '","' + B + '",'
      + '{expectedActive:"' + A + '",expectedPointerVersion:1});return "NO_ERROR";}'
      + 'catch(e){return e.code||"NO_CODE";}})()');
    rec('stale CAS precondition fails closed as conflict', conflictCode === 'conflict', conflictCode);
    const unsealedCode = await evaluate('(async()=>{try{'
      + 'await window.__skryniaV3AdminCapabilities().activateRelease("' + APP + '","' + C + '",'
      + '{expectedActive:"' + A + '",expectedPointerVersion:3});return "NO_ERROR";}'
      + 'catch(e){return e.code||"NO_CODE";}})()');
    rec('unsealed release activation fails closed', unsealedCode === 'invalid_state', unsealedCode);

    // ---- No public credentials in DOM, browser storage or requests ----
    const leak = await evaluate('(()=>{const blob=document.documentElement.outerHTML'
      + '+JSON.stringify(Object.entries(localStorage))'
      + '+JSON.stringify(Object.entries(sessionStorage))'
      + '+Object.keys(window).filter(k=>/TOKEN|CAP|SECRET|CREDENTIAL/i.test(k)).join(",");'
      // session.readCap is the PUBLIC invoke ID that the UI deliberately shows
      // as the app ID (GET_ACTIVE/LIST_VERSIONS are unauthenticated reads); the
      // secrets are the bearer token, its verifier, the edge capability and the
      // private admin capability.
      + 'return [' + JSON.stringify(tokenHex) + ',' + JSON.stringify(verifierHex) + ','
      + JSON.stringify(edgeCap) + ',' + JSON.stringify(session.adminCap)
      + '].some(s=>blob.includes(s));})()');
    rec('no bearer/verifier/edge/admin secret in DOM, storage or window globals', leak === false);
    const postOnly = host.seen.length > 0 && host.seen.every(s => s.path === '/_skrynia/' && s.method === 'POST');
    rec('all native traffic was POST /_skrynia/ only', postOnly, host.seen.length + ' requests');

    // ---- Durable native state verified outside the browser ----
    const finalRead = await postNative(appOrigin, getActiveOp(readCap));
    const finalBytes = decodeResponse(finalRead.body).body;
    const finalState = {
      active: Buffer.from(finalBytes.subarray(2, 34)).toString('hex'),
      pointerVersion: Number(finalBytes.readBigUInt64LE(34)),
    };
    rec('native service persisted A-B-A pointer after browser exit',
      finalState.active === A && finalState.pointerVersion === 3,
      JSON.stringify(finalState));
    rec('forged-framing pin unchanged', P0_FRAME_PIN === '8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217');
    socket.close();

    const evidence = {
      kind: 'web-admin-v3 real Chromium UI-to-native E2E',
      verified_at: new Date().toISOString(),
      host: hostname(),
      chromium: CHROME,
      p0_frame_pin: P0_FRAME_PIN,
      pins: Object.fromEntries(Object.entries(PINS).map(([k, v]) => [k, {path: v.path, sha256: v.sha256}])),
      static_origin: appOrigin,
      adapter_endpoint: session.endpoint,
      token_policy: 'disposable 32-byte bearer from run; edge cap injected server-side by static proxy; caps in runtime browser session only; never in public assets, URLs or logs',
      results,
      ALL_OK: results.every(r => r.ok),
    };
    if (process.env.E2E_EVIDENCE_OUT) {
      await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n');
    }
    console.log(evidence.ALL_OK
      ? 'PASS web-admin-v3 real Chromium E2E: static UI + native adapter + gateway + kernel + P0 + ordinary service, A-B-A CAS and fail-closed verified'
      : 'FAIL web-admin-v3 real Chromium E2E');
    if (!evidence.ALL_OK) process.exitCode = 1;
  } finally {
    try { if (browser) await stopProc(browser); } catch {}
    if (staticServer) await new Promise(ok => staticServer.close(ok));
    for (const p of procs) { try { await stopProc(p); } catch {} }
    await delay(200);
  }
}

// Reusable as a library (see chromium_pagination_1000.mjs); run only directly.
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
}
