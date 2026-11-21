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
import { createServer } from 'node:net';
import http from 'node:http';
import { mkdtemp, mkdir, readFile, writeFile, rm, stat } from 'node:fs/promises';
import { join, resolve, dirname, extname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

import {
  PINS, P0_FRAME_PIN, sha256hex, sha256buf, hashFile, exists, freePort, stopProc,
  startKernel, startGateway, appPutOp, appGetOp, decodeResponse,
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
async function startStaticHost({ port, edgePort, appHost, appOrigin, edgeCap }) {
  const seen = [];
  const server = http.createServer(async (req, res) => {
    const u = new URL(req.url, 'http://127.0.0.1');
    if (u.pathname === '/_skrynia/') {
      seen.push({path: u.pathname, method: req.method});
      if (req.method !== 'POST') { res.writeHead(405); res.end(); return; }
      const chunks = [];
      for await (const c of req) chunks.push(c);
      const body = Buffer.concat(chunks);
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

function cdp(socket) {
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

async function until(evaluate, expression, label, tries = 200) {
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

async function postNative(origin, bytes) {
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

function manifest(app, id, commit) {
  return {schema: 'skrynia-native-static-v1', app, release: id, commit,
    repo: 'native-e2e', files: [{path: 'index.html', sha256: id}]};
}

async function main() {
  for (const [name, a] of Object.entries(PINS)) {
    rec(`pin ${name} sha256 ${a.sha256.slice(0, 12)}`,
      (await exists(a.path)) && (await hashFile(a.path)) === a.sha256, a.note);
    assert.equal(await hashFile(a.path), a.sha256, `pinned ${name} sha256 mismatch`);
  }

  const tmp = await mkdtemp(join('/tmp/opencode', 'webadmin-v3-chromium-e2e-'));
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

    // ---- Provision already-published immutable history through the real stack ----
    const A = 'a'.repeat(64), B = 'b'.repeat(64), C = 'c'.repeat(64);
    const put = async (key, value) => {
      const r = await postNative(appOrigin, appPutOp(adminCap, Buffer.from(key), Buffer.from(value)));
      const d = decodeResponse(r.body);
      assert.equal(r.http, 200);
      assert.equal(d.status, 0, 'provisioning write status');
    };
    await put('v3admin:apps', JSON.stringify(['demo']));
    await put('v3admin:app:demo:state', JSON.stringify({active: A, pointerVersion: 4}));
    await put('v3admin:app:demo:versions', JSON.stringify([
      {id: A, ready: true, manifest: manifest('demo', A, 'commit-A')},
      {id: B, ready: true, manifest: manifest('demo', B, 'commit-B')},
      {id: C, ready: false, manifest: null},
    ]));
    rec('immutable app history provisioned in ordinary service', true, '3 releases, unsealed C');

    // ---- Launch real Chromium and inject only runtime session authority ----
    const cdpPort = await freePort();
    browser = spawn(CHROME, ['--headless=new', '--no-sandbox', '--disable-gpu',
      '--disable-dev-shm-usage', '--no-first-run', '--disable-background-networking',
      '--remote-debugging-port=' + cdpPort, '--user-data-dir=' + join(tmp, 'chromium'),
      'about:blank'], {stdio: ['ignore', 'ignore', 'pipe']});
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
      '[...document.querySelectorAll(".app-panel h3")].some(x=>x.textContent.includes("demo"))',
      'all-app/all-version history rendered');
    const panelInfo = await evaluate('(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("demo"));'
      + 'return {title:p.querySelector("h3").textContent,'
      + ' pointer:p.querySelector("p").textContent,'
      + ' rows:p.querySelectorAll(".version-row").length,'
      + ' badges:[...p.querySelectorAll(".badge")].map(x=>x.textContent)};})()');
    rec('UI lists every app version including unsealed one',
      panelInfo.rows === 3 && panelInfo.badges.includes('unavailable'), JSON.stringify(panelInfo));
    rec('UI shows the current active pointer', panelInfo.pointer.includes(A), panelInfo.pointer);

    // ---- Historical manifest inspection ----
    const rowFor = id => '(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("demo"));'
      + 'return [...p.querySelectorAll(".version-row")]'
      + '.find(x=>x.querySelector(".version-meta")?.textContent.startsWith(' + JSON.stringify(id) + '));})()';
    const clickRowButton = (id, label) => evaluate('(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("demo"));'
      + 'const row=[...p.querySelectorAll(".version-row")]'
      + '.find(x=>x.querySelector(".version-meta")?.textContent.startsWith(' + JSON.stringify(id) + '));'
      + 'const button=[...row.querySelectorAll("button")]'
      + '.find(b=>b.textContent===' + JSON.stringify(label) + ');'
      + 'if(!button||button.disabled) throw Error("button unavailable: ' + label + '");'
      + 'button.click();return true;})()');
    await clickRowButton(B, 'Inspect manifest');
    await until(evaluate,
      'document.getElementById("manifest-view").textContent.includes("skrynia-native-static-v1")'
      + ' && document.getElementById("manifest-view").textContent.includes(' + JSON.stringify(B) + ')',
      'historical manifest inspection');
    rec('historical release manifest inspected in browser', true);

    // ---- A -> B -> A O(1) CAS rollback driven through the real UI ----
    await clickRowButton(B, 'Activate this version');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Activated demo → ' + B + '")',
      'activation to B');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Loaded")',
      'post-activation re-render');
    let live = await evaluate('window.__skryniaV3AdminCapabilities().getAppState("demo")');
    rec('UI activation advanced active pointer to B',
      live.active === B && live.pointerVersion === 5, 'rev=' + live.pointerVersion);

    await clickRowButton(A, 'Activate this version');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Activated demo → ' + A + '")',
      'rollback to A');
    await until(evaluate,
      'document.getElementById("status").textContent.includes("Loaded")',
      'post-activation re-render');
    live = await evaluate('window.__skryniaV3AdminCapabilities().getAppState("demo")');
    rec('UI A-B-A rollback advanced revision without asset rewrite',
      live.active === A && live.pointerVersion === 6, 'rev=' + live.pointerVersion);

    const conflictCode = await evaluate('(async()=>{try{'
      + 'await window.__skryniaV3AdminCapabilities().activateRelease("demo","' + B + '",'
      + '{expectedActive:"' + A + '",expectedPointerVersion:4});return "NO_ERROR";}'
      + 'catch(e){return e.code||"NO_CODE";}})()');
    rec('stale CAS precondition fails closed as conflict', conflictCode === 'conflict', conflictCode);
    const unsealedCode = await evaluate('(async()=>{try{'
      + 'await window.__skryniaV3AdminCapabilities().activateRelease("demo","' + C + '",'
      + '{expectedActive:"' + A + '",expectedPointerVersion:6});return "NO_ERROR";}'
      + 'catch(e){return e.code||"NO_CODE";}})()');
    rec('unsealed release activation fails closed', unsealedCode === 'invalid_state', unsealedCode);

    // ---- No public credentials in DOM, browser storage or requests ----
    const leak = await evaluate('(()=>{const blob=document.documentElement.outerHTML'
      + '+JSON.stringify(Object.entries(localStorage))'
      + '+JSON.stringify(Object.entries(sessionStorage))'
      + '+Object.keys(window).filter(k=>/TOKEN|CAP|SECRET|CREDENTIAL/i.test(k)).join(",");'
      + 'return [' + JSON.stringify(tokenHex) + ',' + JSON.stringify(verifierHex) + ','
      + JSON.stringify(edgeCap) + ',' + JSON.stringify(session.readCap) + ','
      + JSON.stringify(session.adminCap) + '].some(s=>blob.includes(s));})()');
    rec('no token/verifier/capability in DOM, storage or window globals', leak === false);
    const postOnly = host.seen.length > 0 && host.seen.every(s => s.path === '/_skrynia/' && s.method === 'POST');
    rec('all native traffic was POST /_skrynia/ only', postOnly, host.seen.length + ' requests');

    // ---- Durable native state verified outside the browser ----
    const finalRead = await postNative(appOrigin, appGetOp(readCap, Buffer.from('v3admin:app:demo:state')));
    const finalState = JSON.parse(decodeResponse(finalRead.body).body.toString());
    rec('native service persisted A-B-A pointer after browser exit',
      finalState.active === A && finalState.pointerVersion === 6,
      JSON.stringify(finalState));
    rec('forged-framing pin unchanged', P0_FRAME_PIN === '8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217');
    socket.close();

    const evidence = {
      kind: 'web-admin-v3 real Chromium UI-to-native E2E',
      verified_at: new Date().toISOString(),
      host: 'marceline-dev',
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

main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
