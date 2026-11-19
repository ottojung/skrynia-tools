import assert from 'node:assert/strict';
import {createFakeCapabilities} from '../../web-admin-v3/capabilities.js';
import {activationPlan, inspectManifest, loadAllAppStates} from '../../web-admin-v3/apps.js';

function manifest(app, id, commit, files = 2) {
  return {
    schema: 'skrynia-native-static-v1',
    app,
    release: id,
    commit,
    repo: 'ottojung/example',
    files: Array.from({length: files}, (_, i) => ({
      path: i === 0 ? 'index.html' : `assets/app-${i}.js`,
      bytes: 10 + i,
      sha256: 'f'.repeat(64),
    })),
  };
}

const r1 = '1'.repeat(64);
const r2 = '2'.repeat(64);
const r3 = '3'.repeat(64);
const s1 = 'a'.repeat(64);

const caps = createFakeCapabilities({
  apps: {
    alpha: {
      active: r2,
      releases: [
        {id: r1, ready: true, manifest: manifest('alpha', r1, 'oldest', 1)},
        {id: r2, ready: true, manifest: manifest('alpha', r2, 'middle', 500)},
        {id: r3, ready: true, manifest: manifest('alpha', r3, 'newest', 3)},
      ],
    },
    beta: {
      active: null,
      releases: [
        {id: s1, ready: true, manifest: manifest('beta', s1, 'only', 7)},
      ],
    },
  },
});

{
  const names = await caps.listApps();
  assert.deepEqual(names, ['alpha', 'beta'], 'must list all apps');
  const states = await loadAllAppStates(caps);
  assert.equal(states.length, 2);
  const alpha = states.find(s => s.name === 'alpha');
  const beta = states.find(s => s.name === 'beta');
  assert.equal(alpha.active, r2);
  assert.deepEqual(alpha.versions.map(v => v.id), [r1, r2, r3], 'must show all historical versions');
  assert.ok(alpha.versions.every(v => v.usable && v.ready));
  assert.equal(alpha.versions[1].files, 500);
  assert.equal(beta.active, null);
  assert.deepEqual(beta.versions.map(v => v.id), [s1]);
}

{
  const inspected = await inspectManifest(caps, 'alpha', r1);
  assert.equal(inspected.id, r1);
  assert.equal(inspected.commit, 'oldest');
  assert.equal(inspected.files, 1);
  assert.equal(inspected.raw.files[0].path, 'index.html');
}

{
  // Rollback and forward activation: O(1) pointer CAS only, no data migration/copy.
  const before = structuredClone(caps.state);
  const planBack = activationPlan(await caps.getAppState('alpha'), r1);
  assert.equal(planBack.direction, 'switch');
  assert.equal(planBack.expectedActive, r2);
  assert.equal(planBack.expectedPointerVersion, 0);
  const back = await caps.activateRelease('alpha', r1, {expectedActive: r2, expectedPointerVersion: 0});
  assert.equal(back.active, r1);
  assert.equal((await caps.getAppState('alpha')).active, r1);
  // Release records must be unchanged except pointer state.
  const afterBack = caps.state.apps.get('alpha');
  assert.deepEqual(afterBack.releases, before.apps.get('alpha').releases);
  assert.equal(afterBack.pointerVersion, 1);

  await caps.activateRelease('alpha', r3, {expectedActive: r1, expectedPointerVersion: 1});
  assert.equal((await caps.getAppState('alpha')).active, r3);
  await caps.activateRelease('alpha', r1, {expectedActive: r3, expectedPointerVersion: 2});
  assert.equal((await caps.getAppState('alpha')).active, r1);
  await caps.activateRelease('alpha', r2, {expectedActive: r1, expectedPointerVersion: 3});
  assert.equal((await caps.getAppState('alpha')).active, r2);
  const finalAlpha = caps.state.apps.get('alpha');
  assert.deepEqual(finalAlpha.releases, before.apps.get('alpha').releases,
    'historical releases must never be rewritten or migrated');
}

{
  // Initial activation on app with no active pointer.
  await caps.activateRelease('beta', s1, {expectedActive: null, expectedPointerVersion: 0});
  assert.equal((await caps.getAppState('beta')).active, s1);
}

{
  // Conflicts are explicit and pointer remains unchanged.
  assert.equal((await caps.getAppState('alpha')).active, r2);
  await assert.rejects(
    () => caps.activateRelease('alpha', r1, {expectedActive: r3, expectedPointerVersion: 4}),
    err => err.code === 'conflict' && /concurrently/i.test(err.message));
  assert.equal((await caps.getAppState('alpha')).active, r2, 'conflict must not change pointer');

  await assert.rejects(
    () => caps.activateRelease('beta', s1, {expectedActive: null, expectedPointerVersion: 1}),
    err => err.code === 'conflict');
  assert.equal((await caps.getAppState('beta')).active, s1);
}

{
  const bad = '4'.repeat(64);
  await assert.rejects(() => caps.activateRelease('alpha', bad, {expectedActive: r2, expectedPointerVersion: 4}), /does not exist/);
  await assert.rejects(() => inspectManifest(caps, 'alpha', bad), /not found/i);
  await assert.rejects(() => caps.getAppState('missing'), /Unknown application/);
}

{
  // Unready/invalid releases must not be activatable and must be reported.
  const r4 = '5'.repeat(64);
  const r5 = '6'.repeat(64);
  const caps2 = createFakeCapabilities({
    apps: {
      gamma: {
        active: null,
        releases: [
          {id: r4, ready: false, manifest: manifest('gamma', r4, 'partial')},
          {id: r5, ready: true, manifest: {schema: 'skrynia-native-static-v1', app: 'gamma', release: 'wrong', files: []}},
        ],
      },
    },
  });
  const state = await caps2.getAppState('gamma');
  assert.equal(state.versions.length, 2, 'all releases listed even when unavailable');
  assert.ok(state.versions.some(v => v.id === r4 && v.usable === false));
  assert.ok(state.versions.some(v => v.id === r5 && v.usable === false));
  await assert.rejects(() => caps2.activateRelease('gamma', r4, {expectedActive: null, expectedPointerVersion: 0}), /not ready|finalized/i);
  await assert.rejects(() => caps2.activateRelease('gamma', r5, {expectedActive: null, expectedPointerVersion: 0}), /invalid manifest|Cannot activate/i);
  assert.equal((await caps2.getAppState('gamma')).active, null);
}


{
  // A->B->A must not allow a stale request to pass solely because id matches.
  await assert.rejects(
    () => caps.activateRelease('alpha', r3,
      {expectedActive: r2, expectedPointerVersion: 0}),
    err => err.code === 'conflict' && /version/i.test(err.message));
  await assert.rejects(
    () => caps.activateRelease('alpha', r3, {expectedActive: r2}),
    err => err.code === 'invalid_input');
  assert.equal((await caps.getAppState('alpha')).pointerVersion, 4);
}

console.log('v3 app history, manifest inspection, O(1) CAS activation/rollback, and conflict tests passed');
