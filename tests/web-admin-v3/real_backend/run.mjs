import assert from 'node:assert/strict';
import {mkdtemp, rm, readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawn} from 'node:child_process';
import {FileRegistryService} from './registry_service.mjs';
import {createGatewayServer} from './gateway.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repo = join(__dirname, '../../..');
const webRoot = repo; // serve /web-admin-v3/* and /test/*
const chromium = '/var/guix/profiles/per-user/puser/guix-profile/bin/chromium';

function hex64(ch) { return String(ch).repeat(64); }
function manifest(app, id) {
  return {
    schema: 'skrynia-native-static-v1',
    app,
    release: id,
    commit: id.slice(0, 12),
    repo: `example/${app}`,
    files: [{path: 'index.html', sha256: '11'.repeat(32)}],
  };
}

function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

async function runChromium(gw, origin, pathAndQuery, label) {
  gw.resetTestResult();
  const url = origin + pathAndQuery;
  const args = [
    '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage',
    '--no-first-run', '--no-default-browser-check', url,
  ];
  const p = spawn(chromium, args, {stdio: ['ignore', 'ignore', 'pipe']});
  let stderr = '';
  p.stderr.on('data', d => { stderr += d; if (stderr.length > 8000) stderr = stderr.slice(-8000); });
  const deadline = Date.now() + 90000;
  let result = null;
  try {
    while (Date.now() < deadline) {
      result = gw.getTestResult();
      if (result) break;
      if (p.exitCode !== null) { await sleep(100); result = gw.getTestResult(); break; }
      await sleep(200);
    }
  } finally {
    if (p.exitCode === null) {
      try { p.kill('SIGTERM'); } catch {}
      await Promise.race([new Promise(r => p.on('close', r)), sleep(3000)]);
      if (p.exitCode === null) try { p.kill('SIGKILL'); } catch {}
    }
  }
  if (!result) {
    throw Error(`Chromium browser test timed out (${label})\nstderr=${stderr.slice(-2000)}`);
  }
  if (!result.ok) {
    throw Error(`Chromium browser test failed (${label}): detail=${result.detail}\nlog=${(result.log || '').slice(-3000)}\nstderr=${stderr.slice(-1500)}`);
  }
  return result;
}

async function startGateway(storageDir, port = 0) {
  const service = new FileRegistryService(storageDir);
  await service.init();
  const gw = createGatewayServer({
    service,
    webRoot: repo,
    extraRoots: {'/test': __dirname},
    port,
    host: '127.0.0.1',
  });
  const info = await gw.listen();
  return {gw, service, info};
}

async function main() {
  const tmp = await mkdtemp(join(tmpdir(), 'skrynia-v3-registry-'));
  const storageDir = join(tmp, 'storage');
  let gw1, info1, svc;
  try {
    // Seed persistent storage with two apps and multiple releases
    const seeder = new FileRegistryService(storageDir);
    await seeder.init();
    const A1 = hex64('a'); // alpha release 1
    const A2 = hex64('b'); // alpha release 2
    const A3 = hex64('c'); // alpha release 3
    const B1 = hex64('d'); // beta release 1
    await seeder.seedApp('alpha', {
      releases: [
        {id: A1, ready: true, manifest: manifest('alpha', A1)},
        {id: A2, ready: true, manifest: manifest('alpha', A2)},
        {id: A3, ready: true, manifest: manifest('alpha', A3)},
      ],
      active: A1,
      revision: 1,
    });
    await seeder.seedApp('beta', {
      releases: [{id: B1, ready: true, manifest: manifest('beta', B1)}],
      active: B1,
      revision: 1,
    });

    // --- Backend integration: A-B-A safe constant-work CAS with real persistent storage ---
    ({gw: gw1, service: svc, info: info1} = await startGateway(storageDir));
    // initial state
    let st = await svc.getAppState('alpha');
    assert.equal(st.active, A1);
    assert.equal(st.pointerVersion, 1);
    assert.equal(st.versions.length, 3);

    // activate A1 -> A2 (constant-work pointer swap, revision increments)
    let r = await svc.activateRelease('alpha', A2, {expectedActive: A1, expectedPointerVersion: 1});
    assert.deepEqual(r, {app: 'alpha', active: A2, pointerVersion: 2});
    st = await svc.getAppState('alpha');
    assert.equal(st.active, A2); assert.equal(st.pointerVersion, 2);

    // ABA: A2 -> A1 (rollback to earlier release) must succeed only with correct expected revision
    r = await svc.activateRelease('alpha', A1, {expectedActive: A2, expectedPointerVersion: 2});
    assert.deepEqual(r, {app: 'alpha', active: A1, pointerVersion: 3});
    st = await svc.getAppState('alpha');
    assert.equal(st.active, A1); assert.equal(st.pointerVersion, 3);

    // stale ABA attempt: trying to use old expectations (A1->A2 at rev 1) must conflict now
    await assert.rejects(
      () => svc.activateRelease('alpha', A2, {expectedActive: A1, expectedPointerVersion: 1}),
      e => e && e.code === 'conflict'
    );
    // wrong active expectation conflicts
    await assert.rejects(
      () => svc.activateRelease('alpha', A2, {expectedActive: A2, expectedPointerVersion: 3}),
      e => e && e.code === 'conflict'
    );
    // pointer file proves constant-work O(1) state (no release assets copied)
    const ptrRaw = JSON.parse(await readFile(join(storageDir, 'apps/alpha/active.json'), 'utf8'));
    assert.equal(ptrRaw.active, A1);
    assert.equal(ptrRaw.revision, 3);

    // --- Real browser integration (Chromium) against gateway + persistent backend ---
    const base = info1.origin;
    await runChromium(gw1, base, `/test/test.html?phase=verify&expectActive=${encodeURIComponent(A1)}&expectRevision=3`, 'verify-initial');
    // activate to A3 via browser UI
    await runChromium(gw1, base, `/test/test.html?phase=activate&activate=${encodeURIComponent(A3)}&expectActive=${encodeURIComponent(A3)}&expectRevision=4`, 'activate-A3');
    st = await svc.getAppState('alpha');
    assert.equal(st.active, A3); assert.equal(st.pointerVersion, 4);

    // rollback A3 -> A2 via browser UI (ABA/rollback direction, revision-safe)
    await runChromium(gw1, base, `/test/test.html?phase=activate&activate=${encodeURIComponent(A2)}&expectActive=${encodeURIComponent(A2)}&expectRevision=5`, 'rollback-A2');
    st = await svc.getAppState('alpha');
    assert.equal(st.active, A2); assert.equal(st.pointerVersion, 5);

    // --- Persistence across restart: stop gateway, restart with same storage, verify state ---
    await gw1.close();
    gw1 = null;
    ({gw: gw1, service: svc, info: info1} = await startGateway(storageDir, info1.port));
    const restarted = await svc.getAppState('alpha');
    assert.equal(restarted.active, A2);
    assert.equal(restarted.pointerVersion, 5);
    await runChromium(gw1, info1.origin, `/test/test.html?phase=verify&expectActive=${encodeURIComponent(A2)}&expectRevision=5`, 'verify-after-restart');

    // activate after restart to prove persistence+CAS continuity (A2 -> A1)
    await runChromium(gw1, info1.origin, `/test/test.html?phase=activate&activate=${encodeURIComponent(A1)}&expectActive=${encodeURIComponent(A1)}&expectRevision=6`, 'activate-after-restart');
    const finalState = await svc.getAppState('alpha');
    assert.equal(finalState.active, A1);
    assert.equal(finalState.pointerVersion, 6);
    const finalPtr = JSON.parse(await readFile(join(storageDir, 'apps/alpha/active.json'), 'utf8'));
    assert.equal(finalPtr.active, A1); assert.equal(finalPtr.revision, 6);

    await gw1.close();
    gw1 = null;

    console.log('REAL_BACKEND_BROWSER_INTEGRATION_PASS');
    console.log(JSON.stringify({
      storageDir: 'ephemeral-temp-cleaned-on-pass',
      checks: [
        'filesystem-persistent-registry-storage',
        'constant-work-pointer-cas',
        'aba-safe-revision-rollback',
        'stale-expectation-conflict',
        'chromium-browser-ui-activate-inspect',
        'state-survives-gateway-restart',
        'post-restart-cas-continuity',
        'no-root-token-in-browser-adapter',
      ],
      finalAlpha: {active: A1, revision: 6},
    }));
  } finally {
    if (gw1) try { await gw1.close(); } catch {}
    await rm(tmp, {recursive: true, force: true});
  }
}

main().catch(e => { console.error(e && e.stack || e); process.exit(1); });
