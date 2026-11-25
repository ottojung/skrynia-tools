#!/usr/bin/env node
// Real Chromium proof that the shipped web-admin UI lists 1000 sealed ready
// versions through the paged native wire (MAX_PAGE=64), with fail-closed
// checks in the same browser. NO simulator and NO fake adapter for the real
// stack: the static app is hosted on an independent disposable origin, its
// /_skrynia/ path is reverse-proxied to the trusted edge gateway, and the UI
// drives the real pinned kernel + dumb P0 + ordinary paged-index P1 service.
//
// Prebuilt compiled artifacts only (no Cargo/WASM build). No host-specific
// paths or digests are baked in: exactly five artifact path+SHA-256 env pairs
// must be supplied (or a verified local manifest via SKRYNIA_E2E_MANIFEST),
// together with runner provenance. CI reuses the compiled-stack artifact
// downloaded by the same workflow; local runs supply the pairs explicitly.

import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { hostname, tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

import {
  PINS, P0_FRAME_PIN, sha256buf, hashFile, exists, freePort, stopProc,
  startKernel, startGateway, decodeResponse, putId, putBlob,
} from '../run.mjs';
import {
  startStaticHost, cdpTarget, cdp, until, postNative, stageOp, sealOp, getActiveOp,
} from '../chromium.mjs';

const CHROME = process.env.CHROME_BIN || 'chromium';
const PAGE_LIMIT = 64;
const VERSION_COUNT = 1000;
const sha = b => createHash('sha256').update(Buffer.from(b)).digest();

// The five artifacts are always externally supplied; this proof never selects
// a host-specific binary. run.mjs's PINS read the same env pairs at import.
const PIN_ENV = {
  kernel: ['SKRYNIA_KERNEL_BIN', 'SKRYNIA_KERNEL_SHA256'],
  p0: ['SKRYNIA_P0_WASM', 'SKRYNIA_P0_SHA256'],
  service: ['SKRYNIA_P1_WASM', 'SKRYNIA_P1_SHA256'],
  echo: ['SKRYNIA_ECHO_GUEST', 'SKRYNIA_ECHO_SHA256'],
  gateway: ['SKRYNIA_EDGE_GATEWAY_BIN', 'SKRYNIA_EDGE_GATEWAY_SHA256'],
};
const SHA256 = /^[0-9a-f]{64}$/;

function runnerProvenance() {
  const p = {
    host: hostname(),
    os: process.env.RUNNER_OS || process.platform,
    arch: process.arch,
    runId: process.env.GITHUB_RUN_ID || null,
    repository: process.env.GITHUB_REPOSITORY || null,
    gitHead: process.env.GITHUB_SHA || process.env.SKRYNIA_GIT_HEAD || null,
    runProvenance: process.env.SKRYNIA_RUN_PROVENANCE || null,
  };
  if (!p.runId && !p.runProvenance) {
    throw Error('runner provenance required: set SKRYNIA_RUN_PROVENANCE for local runs');
  }
  return p;
}

async function resolvePins() {
  const manifest = process.env.SKRYNIA_E2E_MANIFEST;
  if (manifest) {
    const doc = JSON.parse(await readFile(manifest, 'utf8'));
    for (const [name, [pathEnv, shaEnv]] of Object.entries(PIN_ENV)) {
      const entry = doc[name];
      if (!entry || typeof entry.path !== 'string' ||
          !SHA256.test(String(entry.sha256 || '').toLowerCase())) {
        throw Error('verified manifest missing valid ' + name + ' path+sha256');
      }
      process.env[pathEnv] = entry.path;
      process.env[shaEnv] = String(entry.sha256).toLowerCase();
    }
    return {source: 'verified-manifest', manifest};
  }
  const missing = Object.entries(PIN_ENV)
    .filter(([, [pathEnv, shaEnv]]) =>
      !process.env[pathEnv] || !SHA256.test(String(process.env[shaEnv] || '').toLowerCase()))
    .map(([name]) => name);
  if (missing.length) {
    throw Error('explicit artifact path+sha256 env pairs required; missing/invalid: ' + missing.join(', '));
  }
  return {source: 'explicit-env-pairs', manifest: null};
}

const results = [];
function rec(name, ok, detail = '') {
  results.push({name, ok: Boolean(ok), detail: String(detail)});
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' :: ' + detail : ''}`);
  return ok;
}

// Extract op + exclusive tail cursor + wire limit from a listing frame,
// without retaining any capability bytes.
function parseListingFrame(buf) {
  const b = Buffer.from(buf);
  if (b.length < 4 || b[0] !== 1 || (b[1] !== 12 && b[1] !== 13 && b[1] !== 14)) return null;
  let at = 2;
  if (b[1] === 12 || b[1] === 13) at += 2 + b.readUInt16LE(at);
  if (at >= b.length) return null;
  const flag = b[at++];
  let after = null;
  if (flag === 1) { after = b.subarray(at, at + 32); at += 32; }
  if (at + 2 > b.length) return null;
  return {op: b[1], limit: b.readUInt16LE(at), after: after && after.toString('hex')};
}

async function main() {
  const pinSource = await resolvePins();
  const provenance = runnerProvenance();
  for (const [name, [pathEnv, shaEnv]] of Object.entries(PIN_ENV)) {
    const path = process.env[pathEnv];
    const sha256 = String(process.env[shaEnv]).toLowerCase();
    PINS[name].path = path;
    PINS[name].sha256 = sha256;
    const got = (await exists(path)) ? await hashFile(path) : 'MISSING';
    rec(`pin ${name} sha256 ${sha256.slice(0, 12)}`, got === sha256, path);
    assert.equal(got, sha256, `pinned ${name} sha256 mismatch`);
  }

  const tmp = await mkdtemp(join(tmpdir(), 'webadmin-v3-chromium-1000-'));
  const procs = [];
  let staticServer = null;
  let browser = null;
  try {
    const token = randomBytes(32);
    const tokenHex = Buffer.from(token).toString('hex');
    const verifierHex = sha256hex(token);
    const opCap = sha256buf(Buffer.from('webadmin-v3-1000-operator'));
    const adminCap = sha256buf(Buffer.from('webadmin-v3-1000-admin'));
    const readCap = sha256buf(adminCap);
    const edgeCap = 'webadmin-v3-1000-' + randomBytes(16).toString('hex');

    const kernelPort = await freePort();
    const kernel = await startKernel({base: join(tmp, 'kernel'), port: kernelPort,
      token, opCap, logs: 'kernel.log'});
    procs.push(kernel.proc);
    rec('paged-index kernel + dumb P0 + paged-index P1 booted', kernel.proc.exitCode === null,
      `bind 127.0.0.1:${kernelPort}`);

    const staticPort = await freePort();
    const appHost = `127.0.0.1:${staticPort}`;
    const appOrigin = `http://${appHost}`;
    const edgePort = await freePort();
    const gw = await startGateway({dir: join(tmp, 'edge-gw'), port: edgePort, kernelPort,
      token, edgeCap, logs: 'gateway.log', appHost, appOrigin});
    procs.push(gw.proc);
    rec('trusted edge gateway booted with server-only token', gw.proc.exitCode === null,
      `bind 127.0.0.1:${edgePort}`);

    const listings = [];
    const seen = [];
    const host = await startStaticHost({port: staticPort, edgePort, appHost, appOrigin, edgeCap,
      onRequest: (body, meta) => {
        seen.push(meta);
        const f = parseListingFrame(body);
        if (f) listings.push(f);
      }});
    staticServer = host.server;
    rec('independent disposable static origin serves shipped admin app', true, appOrigin);

    const APP = Buffer.from(readCap).toString('hex');
    const nativeOp = async (frame, label) => {
      const r = await postNative(appOrigin, frame);
      assert.equal(r.http, 200, label + ' http');
      const d = decodeResponse(r.body);
      assert.equal(d.status, 0, label + ' native status err=' + d.body[0]);
      return d.body;
    };

    // ---- Provision 1000 sealed ready versions through real native opcodes ----
    const code0 = await readFile(PINS.echo.path);
    const baseHash = sha(code0).toString('hex');
    await nativeOp(Buffer.concat([Buffer.from([0x01, 0x0f]), putId(opCap), putId(adminCap),
      putBlob(code0)]), 'register');
    const digests = [baseHash];
    for (let n = 1; n < VERSION_COUNT; n += 1) {
      const code = Buffer.from('chromium-1000-ready-' + String(n).padStart(6, '0'));
      const digest = sha(code).toString('hex');
      assert.equal((await nativeOp(stageOp(adminCap, code), 'stage ' + n)).toString('hex'), digest);
      await nativeOp(sealOp(adminCap, Buffer.from(digest, 'hex')), 'seal ' + n);
      digests.push(digest);
    }
    digests.sort(); // SHA-256 content hashes are indexed in raw byte order
    rec('compiled P1 provisioned with 1000 sealed ready versions', digests.length === VERSION_COUNT,
      VERSION_COUNT + ' distinct sha256 content hashes');

    // ---- Launch real Chromium with only runtime session authority ----
    const cdpPort = await freePort();
    browser = spawn(CHROME, ['--headless=new', '--no-sandbox', '--disable-gpu',
      '--disable-dev-shm-usage', '--no-first-run', '--disable-background-networking',
      '--remote-debugging-port=' + cdpPort, '--user-data-dir=' + join(tmp, 'chromium'),
      'about:blank'], {stdio: ['ignore', 'ignore', 'pipe']});
    let chromeStderr = '';
    browser.stderr.on('data', c => { chromeStderr += c.toString(); });
    const wsUrl = await cdpTarget(cdpPort);
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
      'document.readyState==="complete" && !!window.__skryniaV3AdminCapabilities',
      'static app loaded with native adapter');
    rec('real Chromium loaded the shipped native adapter', true, CHROME);
    await evaluate('window.confirm=()=>true;'
      + 'document.getElementById("vault-secret").value="e2e-access-secret";'
      + 'document.getElementById("unlock-form").requestSubmit();');
    await until(evaluate, 'document.getElementById("workspace").hidden===false', 'vault unlock');

    // ---- Browser DOM proof: every one of the 1000 ready versions is listed ----
    await until(evaluate, '(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("' + APP + '"));'
      + 'return p && p.querySelectorAll(".version-row").length===' + VERSION_COUNT + ';})()',
    '1000 ready versions rendered in the shipped UI', 600);
    const panel = await evaluate('(()=>{const p=[...document.querySelectorAll(".app-panel")]'
      + '.find(x=>x.querySelector("h3")?.textContent.includes("' + APP + '"));'
      + 'const rows=[...p.querySelectorAll(".version-row .version-meta")]'
      + '.map(x=>x.childNodes[0].textContent);'
      + 'return {title:p.querySelector("h3").textContent,rows:rows.length,unique:new Set(rows).size,'
      + 'ready:p.querySelectorAll(".badge.ready").length};})()');
    rec('shipped UI renders all 1000 sealed versions (DOM proof)',
      panel.rows === VERSION_COUNT && panel.unique === VERSION_COUNT
      && panel.ready === VERSION_COUNT && panel.title.includes('1000 release(s)'),
      JSON.stringify(panel));

    // ---- Wire proof: 64-entry pages with strictly advancing tail keys ----
    const versionListings = listings.filter(f => f.op === 13);
    rec('every LIST_VERSIONS request used the confirmed 64-entry wire cap',
      versionListings.length >= Math.ceil(VERSION_COUNT / PAGE_LIMIT)
      && versionListings.every(f => f.limit === PAGE_LIMIT),
      versionListings.length + ' LIST_VERSIONS requests, limits='
      + [...new Set(versionListings.map(f => f.limit))].join(','));
    let cursorsAdvance = true;
    let previousCursor = null;
    let cursorCount = 0;
    for (const f of versionListings) {
      if (!f.after) continue; // first page has no cursor
      cursorCount += 1;
      if (previousCursor !== null && !(f.after > previousCursor)) {
        cursorsAdvance = false;
        break;
      }
      previousCursor = f.after;
    }
    rec('tail keys are raw-byte strictly advancing 32-byte index keys',
      cursorsAdvance && cursorCount === Math.ceil(VERSION_COUNT / PAGE_LIMIT) - 1,
      cursorCount + ' tail cursors');
    rec('all native traffic was POST /_skrynia/ only',
      seen.length > 0 && seen.every(s => s.path === '/_skrynia/' && s.method === 'POST'),
      seen.length + ' requests');

    // ---- Fail-closed inside the same real Chromium, using the shipped module ----
    const failCodes = await evaluate('(async()=>{'
      + 'const {createNativeCapabilities}=await import("/native_capabilities.js");'
      + 'const key=n=>{const b=new Uint8Array(32);b[31]=n;return b;};'
      + 'const frame=hs=>{const b=new Uint8Array(2+hs.length*32);'
      + 'new DataView(b.buffer).setUint16(0,hs.length,true);'
      + 'hs.forEach((h,i)=>b.set(new Uint8Array(h),2+i*32));'
      + 'const e=new Uint8Array(6+b.length);e[0]=1;'
      + 'new DataView(e.buffer).setUint32(2,b.length,true);e.set(b,6);return e.buffer;};'
      + 'const run=async hs=>{const caps=createNativeCapabilities({'
      + 'endpoint:"http://127.0.0.1:1/_skrynia/",readCap:"11".repeat(32),'
      + 'adminCap:"22".repeat(32)},async()=>({status:200,'
      + 'arrayBuffer:async()=>frame(hs)}));'
      + 'try{await caps.listApps();return "NO_ERROR";}catch(e){return e.code;}};'
      + 'const over=Array.from({length:65},(_,i)=>key(i+1));'
      + 'return [await run(over),await run([key(2),key(1)])];})()');
    rec('overlimit and unsorted pages fail closed in Chromium',
      Array.isArray(failCodes) && failCodes.every(c => c === 'invalid_data'),
      JSON.stringify(failCodes));

    const leak = await evaluate('(()=>{const blob=document.documentElement.outerHTML'
      + '+JSON.stringify(Object.entries(localStorage))'
      + '+JSON.stringify(Object.entries(sessionStorage));'
      + 'return [' + JSON.stringify(tokenHex) + ',' + JSON.stringify(verifierHex) + ','
      + JSON.stringify(edgeCap) + ',' + JSON.stringify(session.adminCap)
      + '].some(s=>blob.includes(s));})()');
    rec('no bearer/verifier/edge/admin secret in DOM or storage', leak === false);

    const active = decodeResponse((await postNative(appOrigin, getActiveOp(readCap))).body).body;
    rec('active pointer unchanged by 1000-version listing',
      active.subarray(2, 34).toString('hex') === baseHash && active.readBigUInt64LE(34) === 1n);
    rec('forged-framing pin unchanged',
      P0_FRAME_PIN === '8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217');
    socket.close();

    const evidence = {
      kind: 'web-admin-v3 real Chromium 1000-ready-version pagination proof',
      verified_at: new Date().toISOString(),
      host: provenance.host,
      provenance,
      pin_source: pinSource,
      chromium: CHROME,
      page_limit: PAGE_LIMIT,
      version_count: VERSION_COUNT,
      p0_frame_pin: P0_FRAME_PIN,
      pins: Object.fromEntries(Object.entries(PINS).map(([k, v]) => [k, {path: v.path, sha256: v.sha256}])),
      static_origin: appOrigin,
      listing_requests: versionListings.length,
      results,
      ALL_OK: results.every(r => r.ok),
    };
    if (process.env.E2E_EVIDENCE_OUT) {
      await writeFile(process.env.E2E_EVIDENCE_OUT, JSON.stringify(evidence, null, 2) + '\n');
    }
    console.log(evidence.ALL_OK
      ? 'PASS real Chromium 1000-version pagination: shipped UI listed all 1000 sealed releases in 64-entry pages'
      : 'FAIL real Chromium 1000-version pagination');
    if (!evidence.ALL_OK) process.exitCode = 1;
  } finally {
    try { if (browser) await stopProc(browser); } catch {}
    if (staticServer) await new Promise(ok => staticServer.close(ok));
    for (const p of procs) { try { await stopProc(p); } catch {} }
    await delay(200);
    await rm(tmp, {recursive: true, force: true}).catch(() => {});
  }
}

function sha256hex(b) { return createHash('sha256').update(Buffer.from(b)).digest('hex'); }

main().catch(e => { console.error(e.stack || String(e)); process.exit(1); });
