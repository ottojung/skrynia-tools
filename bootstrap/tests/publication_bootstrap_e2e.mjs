#!/usr/bin/env node
// Disposable REAL compiled publication + real P0/P1/edge + Chromium bootstrap fixture.
// No rebuild, no production config/TLS/DNS/credentials, no real secrets.
import assert from 'node:assert/strict';
import {createHash, randomBytes} from 'node:crypto';
import {spawn, spawnSync} from 'node:child_process';
import http from 'node:http';
import {mkdir, mkdtemp, readFile, readdir, rm, writeFile, chmod} from 'node:fs/promises';
import {join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {pathToFileURL} from 'node:url';
import {setTimeout as delay} from 'node:timers/promises';

import {install, KIT_TOOLS_PIN} from '../install.mjs';
import {
  PINS, sha256hex, sha256buf, hashFile, exists, freePort, stopProc,
  startKernel, startGateway, registerOp, decodeResponse, gatewayFetch,
} from '../../tests/web-admin-v3/real_native_e2e/run.mjs';
import {cdpTarget, cdp, until} from '../../tests/web-admin-v3/real_native_e2e/chromium.mjs';

const KIT = process.env.SKRYNIA_KIT || '/workspace/skrynia-exact-kit-73d8fa1';
const NGINX = process.env.NGINX_BIN
  || '/gnu/store/db9fbd8i7dxn3k2rs7rll52d1gxx4wsy-nginx-1.28.0/sbin/nginx';
const CHROME = process.env.CHROMIUM_BIN
  || process.env.CHROME_BIN
  || '/var/guix/profiles/per-user/puser/guix-profile/bin/chromium';
const PW = process.env.SKRYNIA_PLAYWRIGHT_MODULE
  || '/workspace/skrynia-tools/tests/browser/node_modules/playwright/index.mjs';

const sha = b => createHash('sha256').update(b).digest('hex');
const results = [];
function rec(name, ok, detail = '') {
  const pass = Boolean(ok);
  results.push({name, ok: pass, detail: String(detail)});
  console.log((pass ? 'PASS ' : 'FAIL ') + name + (detail ? ' :: ' + detail : ''));
  if (!pass) throw new Error('FAIL: ' + name + (detail ? ' :: ' + detail : ''));
}

async function waitHttp(url, tries = 240) {
  for (let i = 0; i < tries; i++) {
    try {
      const res = await fetch(url, {signal: AbortSignal.timeout(1000)});
      // nginx readiness: any HTTP response means the server is up; 404 is fine
      // before published files exist.
      if (res.status > 0) { await res.arrayBuffer().catch(() => {}); return true; }
    } catch {}
    await delay(50);
  }
  return false;
}
async function getBytes(url) {
  const res = await fetch(url, {signal: AbortSignal.timeout(5000)});
  return {status: res.status, bytes: Buffer.from(await res.arrayBuffer())};
}
async function waitPost(url, tries = 240) {
  for (let i = 0; i < tries; i++) {
    try {
      const res = await fetch(url, {method: 'POST', signal: AbortSignal.timeout(1000)});
      if (res.status > 0) { await res.arrayBuffer().catch(() => {}); return true; }
    } catch {}
    await delay(50);
  }
  return false;
}
function callHelper(wrapper, payload) {
  const cp = spawnSync(wrapper, [], {input: JSON.stringify(payload), encoding: 'utf8', timeout: 30000});
  if (cp.error || cp.status !== 0) throw Error('helper failed: ' + (cp.error?.message || cp.status));
  const out = JSON.parse(cp.stdout);
  if (out.ok !== true) throw Error('helper reported failure');
  return out;
}

async function main() {
  const tmp = await mkdtemp(join(tmpdir(), 'bootstrap-pub-e2e-'));
  const procs = [];
  let browser = null;
  let nginxProc = null;
  let allOk = false;
  try {
    // Fail immediately on wrong kit digest - no "continue on wrong binary".
    // Exact-head qualification may supply a different compiled kit digest via
    // SKRYNIA_KIT_<NAME>_SHA256; the historical defaults stay fail-closed and
    // any override must itself be a 64-hex digest.
    const kitDigest = (name, def) => {
      const v = process.env['SKRYNIA_KIT_' + name.toUpperCase().replaceAll('-', '_') + '_SHA256'] || def;
      if (!/^[0-9a-f]{64}$/i.test(v)) throw Error('invalid kit digest override for ' + name);
      return v.toLowerCase();
    };
    for (const [name, path, expected] of [
      ['kernel', join(KIT, 'kernel-release'), kitDigest('kernel', 'a1dc4119d360436cffa35ba19416d8d5c90bfbf6237fcc2772b46a8c57459fd1')],
      ['publication-root', join(KIT, 'publication-root.wasm'), kitDigest('publication-root', '35fe0f955019bc24f40c32c3a3813db25f5a25ee729acc6caed57f20c81da65a')],
      ['provider', join(KIT, 'provider'), kitDigest('provider', '6bedbc1a8f130441ef3d5247cfd82ac62a01cba4e0d07c2ebb6d6a2b64854c0b')],
    ]) {
      const got = sha(await readFile(path));
      rec('exact-kit-' + name, got === expected, got);
    }
    // Real P1 admin stack pins are verified by startKernel/startGateway helpers.
    for (const [name, pin] of Object.entries(PINS)) {
      const got = (await exists(pin.path)) ? await hashFile(pin.path) : 'MISSING';
      rec(`pin-${name}`, got === pin.sha256, got.slice(0, 12));
    }

    // --- Real P0/P1/edge admin stack ---
    const p1Token = randomBytes(32);
    const opCap = randomBytes(32);
    const adminCap = randomBytes(32);
    const adminHex = adminCap.toString('hex');
    const iid = sha256buf(adminCap);
    const app = iid.toString('hex'); // native readCap / app id
    const echo = await readFile(PINS.echo.path);

    const nginxPort = await freePort();
    const origin = `http://127.0.0.1:${nginxPort}`;
    const appHost = `127.0.0.1:${nginxPort}`;

    const kernelPort = await freePort();
    const kernel = await startKernel({base: join(tmp, 'admin-kernel'), port: kernelPort,
      token: p1Token, opCap, logs: 'kernel.log'});
    procs.push(kernel.proc);
    rec('admin-kernel-ready', await waitPost(`http://127.0.0.1:${kernelPort}/version`), `127.0.0.1:${kernelPort}`);

    const edgePort = await freePort();
    const edgeCap = 'bootstrap-e2e-' + randomBytes(16).toString('hex');
    const gw = await startGateway({dir: join(tmp, 'edge'), port: edgePort,
      kernelPort, token: p1Token, edgeCap, logs: 'gateway.log', appHost, appOrigin: origin});
    procs.push(gw.proc);
    rec('admin-gateway-ready', await waitHttp(`http://${gw.host}/version`), gw.host);

    // Register an app out of band so real admin authority has something to list.
    const transportEndpoint = `http://${gw.host}/_skrynia/`;
    const rawFetch = gatewayFetch(gw);
    const regResp = await rawFetch(transportEndpoint, {
      method: 'POST',
      headers: {'Content-Type': 'application/octet-stream'},
      body: registerOp(opCap, adminCap, echo),
    });
    const registered = decodeResponse(Buffer.from(await regResp.arrayBuffer()));
    rec('admin-app-registered', registered.status === 0, `status ${registered.status}`);

    // --- Compiled publication root/provider + nginx ---
    const operator = join(tmp, 'operator'), mailbox = join(tmp, 'publication-user', 'publication-requests');
    const source = join(tmp, 'source-root'), nginxRoot = join(tmp, 'nginx-root'), nginxWork = join(tmp, 'nginx-work');
    for (const d of [operator, mailbox, source, nginxRoot, nginxWork]) await mkdir(d, {recursive: true});
    await mkdir(join(operator, 'security'), {recursive: true});
    const pubToken = randomBytes(32);
    await writeFile(join(operator, 'security', 'program0.sha256'), Buffer.from(sha256hex(pubToken), 'hex'));
    const tokenFile = join(tmp, 'pub-token.hex');
    await writeFile(tokenFile, pubToken.toString('hex'), {mode: 0o600});
    await chmod(tokenFile, 0o600);

    const providerPort = await freePort(), pubKernelPort = await freePort();
    const provider = spawn(join(KIT, 'provider'), [], {env: {...process.env,
      PUBLICATION_PROVIDER_BIND: `127.0.0.1:${providerPort}`,
      PATH_TO_PUBLICATION_REQUESTS: mailbox,
      PATH_TO_SKRYNIA_DISK: source,
      PATH_TO_NGINX_SKRYNIA_ROOT: nginxRoot}, stdio: 'ignore'});
    procs.push(provider);
    const pubKernel = spawn(join(KIT, 'kernel-release'), [], {env: {...process.env,
      SKRYNIA_ROOT_WASM: join(KIT, 'publication-root.wasm'),
      SKRYNIA_OPERATOR_DIR: operator,
      SKRYNIA_USER_DIR: join(tmp, 'publication-user'),
      SKRYNIA_BIND: `127.0.0.1:${pubKernelPort}`,
      SKRYNIA_SYSCALL_URL: `http://127.0.0.1:${providerPort}`}, stdio: 'ignore'});
    procs.push(pubKernel);

    const conf = join(tmp, 'nginx.conf');
    // nginx serves published static files and reverse-proxies /_skrynia/ to the
    // real trusted edge gateway (server-side edge cap injection, never in browser).
    await writeFile(conf, `worker_processes 1;\nerror_log /dev/null;\npid ${join(nginxWork, 'nginx.pid')};\nevents { worker_connections 64; }\nhttp {\n  types {\n    text/html html;\n    text/css css;\n    text/javascript js mjs;\n    application/json json;\n    application/wasm wasm;\n  }\n  default_type application/octet-stream;\n  access_log off;\n  client_body_temp_path ${join(nginxWork, 'body')};\n  proxy_temp_path ${join(nginxWork, 'proxy')};\n  fastcgi_temp_path ${join(nginxWork, 'fastcgi')};\n  uwsgi_temp_path ${join(nginxWork, 'uwsgi')};\n  scgi_temp_path ${join(nginxWork, 'scgi')};\n  server {\n    listen 127.0.0.1:${nginxPort};\n    server_name localhost;\n    root ${nginxRoot};\n    disable_symlinks on;\n    location = /_skrynia/ {\n      proxy_pass http://127.0.0.1:${edgePort}/_skrynia/;\n      proxy_set_header Host ${appHost};\n      proxy_set_header Origin ${origin};\n      proxy_set_header X-Skrynia-Edge-Capability ${edgeCap};\n      proxy_set_header Content-Type application/octet-stream;\n    }\n    location /apps/ { disable_symlinks off; try_files $uri =404; }\n    location / { try_files $uri =404; }\n  }\n}\n`);
    for (const d of ['body','proxy','fastcgi','uwsgi','scgi']) await mkdir(join(nginxWork, d), {recursive: true});
    const check = spawn(NGINX, ['-t', '-p', nginxWork, '-c', conf], {stdio: 'ignore'});
    await new Promise(r => check.on('exit', r));
    rec('nginx-config-valid', check.exitCode === 0);
    nginxProc = spawn(NGINX, ['-g', 'daemon off;', '-p', nginxWork, '-c', conf], {stdio: 'ignore'});
    procs.push(nginxProc);

    rec('provider-ready', await waitHttp(`http://127.0.0.1:${providerPort}/syscalls_list`));
    rec('pub-kernel-ready', await waitHttp(`http://127.0.0.1:${pubKernelPort}/version`));
    // nginx readiness: 404 is acceptable before files exist.
    rec('nginx-ready', await waitHttp(`${origin}/__not_yet_published__`));

    const helperPath = resolve('bootstrap/native-publish.mjs');
    const wrapper = join(tmp, 'publish-helper.sh');
    await writeFile(wrapper, `#!/bin/sh\nexport SKRYNIA_PUBLICATION_URL=http://127.0.0.1:${pubKernelPort}/\nexport SKRYNIA_PUBLICATION_TOKEN_FILE=${tokenFile}\nexport SKRYNIA_PUBLIC_BASE_URL=${origin}\nexport SKRYNIA_NGINX_ROOT=${nginxRoot}\nexport SKRYNIA_ARTIFACT_ROOT=${source}\nexec node ${helperPath}\n`, {mode: 0o700});
    await chmod(wrapper, 0o700);

    const readCapFile = join(tmp, 'read.cap'), adminCapFile = join(tmp, 'admin.cap');
    await writeFile(readCapFile, app, {mode: 0o600}); // real P1 app id as readCap
    await writeFile(adminCapFile, adminHex, {mode: 0o600});

    process.env.SKRYNIA_ALLOW_LOOPBACK_HTTP = '1';
    process.env.SKRYNIA_ALLOW_NONTEST_ORIGIN = '1';
    process.env.SKRYNIA_NGINX_ROOT = nginxRoot;
    const {link, password, release} = await install({
      kitPin: KIT_TOOLS_PIN, artifactRoot: source,
      origin, app: 'admin-v3', endpoint: '/_skrynia/',
      readCapFile, adminCapFile, adminCapsFile: '',
      publishHelper: wrapper, verifyHelper: wrapper,
      originUrl: new URL(origin),
      endpointUrl: `${origin}/_skrynia/`,
    });
    rec('installer-stable-link', link === `${origin}/apps/admin-v3/unlock.html`, link);

    // Independent served-byte verification (not helper oracle).
    const releaseDir = join(source, 'apps', 'admin-v3', release);
    const manifest = JSON.parse(await readFile(join(releaseDir, 'manifest.json'), 'utf8'));
    const stable = await getBytes(link);
    rec('stable-entry-served', stable.status === 200);
    const stableHash = sha(stable.bytes);
    let allHashes = stableHash === sha(await readFile(join(releaseDir, 'unlock.html')));
    for (const f of manifest.files) {
      const got = await getBytes(`${origin}/apps/admin-v3/${release}/${f.rel}`);
      if (got.status !== 200 || sha(got.bytes) !== f.sha256) allHashes = false;
    }
    rec('independently-verified-served-bytes', allHashes);

    // --- Actual native publish/unpublish through the compiled root/provider ---
    // Rebuild the exact publication roster from the private artifact bytes.
    const names = (await readdir(releaseDir)).filter(n => /\.(js|html|css|json)$/.test(n));
    const entries = await Promise.all(names.map(async n => ({
      n, h: sha(await readFile(join(releaseDir, n))),
    })));
    const pubPayload = {
      app: 'admin-v3', release,
      files: entries.map(({n, h}) => ({rel: `apps/admin-v3/${release}/${n}`, abs: join(releaseDir, n), sha256: h})),
      stable: entries.map(({n, h}) => ({rel: `apps/admin-v3/${n}`, abs: join(releaseDir, n), sha256: h})),
    };
    const undone = callHelper(wrapper, {...pubPayload, action: 'unpublish'});
    rec('native-unpublish-op-ok', undone.action === 'unpublish');
    let absent = true;
    for (const f of [...pubPayload.files, ...pubPayload.stable]) {
      const got = await getBytes(`${origin}/${f.rel}`);
      if (got.status !== 404) absent = false;
    }
    rec('native-unpublish-removed-served-files', absent);
    const repub = callHelper(wrapper, {...pubPayload, action: 'publish'});
    rec('native-republish-op-ok', repub.action === 'publish');
    const reproof = callHelper(wrapper, {...pubPayload, action: 'verify'});
    const wanted = new Map([...pubPayload.files, ...pubPayload.stable].map(f => [f.rel, f.sha256]));
    rec('native-republish-served-bytes', reproof.files.length === wanted.size
      && reproof.files.every(f => f.status === 200 && wanted.get(f.rel) === f.sha256));

    // --- Real Chromium: no authority before/wrong password, real admin after unlock ---
    const {chromium} = await import(pathToFileURL(PW).href);
    browser = await chromium.launch({executablePath: CHROME, headless: true, args: ['--no-sandbox']});
    const context = await browser.newContext();
    const page = await context.newPage();
    const nativePosts = [];
    page.on('request', req => {
      if (req.method() === 'POST' && req.url().includes('/_skrynia/')) nativePosts.push(req.url());
    });

    await page.goto(link, {waitUntil: 'load'});
    rec('chromium-no-session-before-unlock', await page.evaluate(() => !globalThis.__skryniaNativeSession));
    rec('chromium-no-native-post-before-unlock', nativePosts.length === 0, String(nativePosts.length));

    await page.fill('#secret', 'wrong-password');
    await page.click('button[type=submit]');
    await page.waitForFunction(() => /Unlock failed/.test(document.getElementById('status')?.textContent || ''));
    rec('chromium-wrong-password-no-session', await page.evaluate(() => !globalThis.__skryniaNativeSession));
    rec('chromium-wrong-password-no-native-post', nativePosts.length === 0, String(nativePosts.length));

    await page.fill('#secret', password);
    await page.click('button[type=submit]');
    await page.waitForFunction(() => !!globalThis.__skryniaNativeSession
      && /capability adapter loaded/.test(document.getElementById('capability-status')?.textContent || ''), null, {timeout: 30000});

    const session = await page.evaluate(() => globalThis.__skryniaNativeSession);
    rec('chromium-unlocked-scoped-session', session.readCap === app && session.adminCap === adminHex);
    rec('chromium-session-has-no-forbidden-keys', !('token' in session) && !('operator' in session));

    // Real P0/P1/edge success after unlock: actual native admin calls succeed.
    const realResult = await page.evaluate(async (expectedApp) => {
      const caps = globalThis.__skryniaV3AdminCapabilities();
      const apps = await caps.listApps();
      const state = await caps.getAppState(expectedApp);
      return {apps, active: state.active, pointerVersion: state.pointerVersion, versions: state.versions.length};
    }, app);
    rec('chromium-real-listApps', Array.isArray(realResult.apps) && realResult.apps.includes(app), JSON.stringify(realResult.apps));
    rec('chromium-real-getAppState', realResult.pointerVersion === 1 && realResult.versions >= 1, JSON.stringify(realResult));
    rec('chromium-real-native-posts-happened', nativePosts.length > 0, String(nativePosts.length));

    // Reload re-locks: no plaintext capability survives.
    await page.reload({waitUntil: 'load'});
    rec('chromium-reload-relocks', await page.evaluate(() => !globalThis.__skryniaNativeSession
      && !!document.getElementById('unlock-panel')));
    await browser.close(); browser = null;

    console.log('E2E_RELEASE=' + release);
    console.log('E2E_LINK=' + link);
    console.log('E2E_NATIVE_POSTS=' + nativePosts.length);
    allOk = results.every(r => r.ok);
  } finally {
    if (browser) try { await browser.close(); } catch {}
    for (const p of procs) { try { await stopProc(p); } catch {} }
    if (nginxProc) try { nginxProc.kill('SIGTERM'); } catch {}
    await delay(200);
    for (const p of procs) { try { p.kill('SIGKILL'); } catch {} }
    if (allOk) await rm(tmp, {recursive: true, force: true}).catch(() => {});
    else console.error('preserved tmp for diagnosis:', tmp);
  }
  if (!results.every(r => r.ok)) process.exitCode = 1;
}

main().catch(e => {
  console.error('E2E failed: ' + (e.stack || String(e)));
  process.exitCode = 1;
});
