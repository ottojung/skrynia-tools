import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {mkdtemp, mkdir, readFile, rm, writeFile, chmod} from 'node:fs/promises';
import {join, extname} from 'node:path';
import {tmpdir} from 'node:os';
import {pathToFileURL} from 'node:url';
import {install} from '../install.mjs';

const PW = process.env.SKRYNIA_PLAYWRIGHT_MODULE
  || '/workspace/skrynia-tools/tests/browser/node_modules/playwright/index.mjs';
const CHROME = process.env.CHROMIUM_BIN
  || '/var/guix/profiles/per-user/puser/guix-profile/bin/chromium';
const MIME = {'.js': 'text/javascript', '.html': 'text/html', '.css': 'text/css', '.json': 'application/json'};

test('Chromium: scoped native authority exists only after password unlock', async t => {
  const dir = await mkdtemp(join(tmpdir(), 'bootstrap-chromium-'));
  t.after(() => rm(dir, {recursive: true, force: true}));
  const root = join(dir, 'root');
  const nginxRoot = join(dir, 'nginx-root');
  const mailbox = join(dir, 'publication-mailbox');
  for (const p of [root, nginxRoot, mailbox]) {
    await mkdir(p, {mode: 0o700});
    await chmod(p, 0o700);
  }
  await writeFile(join(dir, 'r.cap'), 'a'.repeat(64), {mode: 0o600});
  await writeFile(join(dir, 'a.cap'), 'b'.repeat(64), {mode: 0o600});
  const helperMod = join(dir, 'helper.mjs');
  await writeFile(helperMod, `
let raw = '';
for await (const chunk of process.stdin) raw += chunk.toString('utf8');
const j = JSON.parse(raw);
if (j.action === 'verify') {
  const files = [...j.files, ...(j.stable || [])].map(f => ({rel: f.rel, status: 200, sha256: f.sha256}));
  process.stdout.write(JSON.stringify({ok: true, release: j.release, origin: j.origin, files}) + '\\n');
} else process.stdout.write(JSON.stringify({ok: true, release: j.release, action: j.action}) + '\\n');
`);
  const helper = join(dir, 'helper.sh');
  await writeFile(helper, `#!/bin/sh\nexec node '${helperMod}'\n`);
  await chmod(helper, 0o700);
  const {createHash} = await import('node:crypto');
  const bundleId = createHash('sha256').update(await readFile(new URL('../SHA256SUMS', import.meta.url))).digest('hex');
  const cfg = {
    bundleId, artifactRoot: root, nginxRoot, mailbox,
    origin: 'https://admin.example.test',
    app: 'admin-v3', endpoint: '/_skrynia/',
    readCapFile: join(dir, 'r.cap'), adminCapFile: join(dir, 'a.cap'),
    adminCapsFile: '', publishHelper: helper, verifyHelper: helper,
  };
  const {release, password, link} = await install(cfg);
  const served = join(root, 'apps', 'admin-v3', release);
  const server = createServer(async (req, res) => {
    const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    let rel = null;
    const releasePrefix = `/apps/admin-v3/${release}/`;
    const stablePrefix = '/apps/admin-v3/';
    if (path.startsWith(releasePrefix)) rel = path.slice(releasePrefix.length);
    else if (path.startsWith(stablePrefix)) rel = path.slice(stablePrefix.length);
    const file = rel ? join(served, rel) : null;
    try {
      const bytes = await readFile(file);
      res.writeHead(200, {'Content-Type': MIME[extname(file)] || 'application/octet-stream'});
      res.end(bytes);
    } catch { res.writeHead(404); res.end('no'); }
  });
  await new Promise(r => server.listen(0, '127.0.0.1', r));
  t.after(() => server.close());
  const base = link.replace('https://admin.example.test', `http://127.0.0.1:${server.address().port}`);

  const {chromium} = await import(pathToFileURL(PW).href);
  const browser = await chromium.launch({executablePath: CHROME, headless: true, args: ['--no-sandbox']});
  t.after(() => browser.close());
  const page = await browser.newPage();
  page.on('pageerror', e => { if (process.env.BOOTSTRAP_DEBUG) console.error('PAGEERROR', e.message); });
  page.on('console', m => { if (process.env.BOOTSTRAP_DEBUG) console.error('CONSOLE', m.type(), m.text()); });
  page.on('response', r => { if (process.env.BOOTSTRAP_DEBUG && r.status() >= 400) console.error('HTTP', r.status(), r.url()); });
  let gatewayCalls = 0;
  await page.route('**/_skrynia/**', async route => {
    gatewayCalls += 1;
    await route.fulfill({status: 503, body: ''});
  });

  await page.goto(base, {waitUntil: 'load'});
  const before = await page.evaluate(() => ({
    session: globalThis.__skryniaNativeSession || null,
    factory: typeof globalThis.__skryniaV3AdminCapabilities,
    stored: JSON.stringify(Object.fromEntries(Object.entries(localStorage))),
  }));
  assert.equal(before.session, null, 'no native session before unlock');
  assert.equal(before.factory, 'undefined', 'no capability factory before unlock');
  assert.equal(before.stored.includes('a'.repeat(64)), false);
  assert.equal(before.stored.includes(password), false);

  await page.fill('#secret', 'wrong-password');
  await page.click('button[type=submit]');
  await page.waitForFunction(() => /Unlock failed/.test(document.getElementById('status')?.textContent || ''));
  const afterBad = await page.evaluate(() => ({
    session: globalThis.__skryniaNativeSession || null,
    factory: typeof globalThis.__skryniaV3AdminCapabilities,
  }));
  assert.equal(afterBad.session, null, 'wrong password must not install a session');
  assert.equal(afterBad.factory, 'undefined');
  assert.equal(gatewayCalls, 0, 'wrong password must not permit authority calls');

  await page.fill('#secret', password);
  await page.click('button[type=submit]');
  await page.waitForFunction(() => !!globalThis.__skryniaNativeSession
    && /capability adapter loaded|fail-closed/.test(document.getElementById('capability-status')?.textContent || ''));
  const afterGood = await page.evaluate(() => ({
    session: globalThis.__skryniaNativeSession,
    status: document.getElementById('capability-status')?.textContent || '',
    unlockGone: !document.getElementById('unlock-panel'),
    url: globalThis.location.href,
  }));
  assert.equal(afterGood.session.readCap, 'a'.repeat(64));
  assert.equal(afterGood.session.adminCap, 'b'.repeat(64));
  assert.equal(afterGood.session.endpoint, 'https://admin.example.test/_skrynia/');
  assert.match(afterGood.status, /capability adapter loaded/);
  assert.equal(afterGood.unlockGone, true, 'same-document admin startup');
  assert.match(afterGood.url, /unlock\.html$/, 'no navigation that would drop the in-memory session');

  // Real capability call attempt through the injected native adapter.
  await page.evaluate(async () => {
    const factory = globalThis.__skryniaV3AdminCapabilities;
    const api = factory();
    try { await api.listApps(); } catch { /* gateway intentionally fails this fixture */ }
  });
  await page.waitForFunction(() => performance.getEntriesByType('resource')
    .some(r => r.name.includes('/_skrynia/')) || true);
  assert.ok(gatewayCalls > 0, 'unlocked adapter must attempt a same-origin native POST');

  const storedAfter = await page.evaluate(() => JSON.stringify(Object.fromEntries(Object.entries(localStorage))));
  assert.equal(storedAfter.includes(password), false, 'password must not be persisted');

  // Reload returns to locked state; no plaintext capability survives.
  await page.reload({waitUntil: 'load'});
  const reloaded = await page.evaluate(() => ({
    session: globalThis.__skryniaNativeSession || null,
    factory: typeof globalThis.__skryniaV3AdminCapabilities,
    unlock: !!document.getElementById('unlock-panel'),
  }));
  assert.equal(reloaded.session, null);
  assert.equal(reloaded.factory, 'undefined');
  assert.equal(reloaded.unlock, true);
});
