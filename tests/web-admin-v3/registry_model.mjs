import assert from 'node:assert/strict';
import {encodeActivePointer, listAppIdsFromKeys, normalizeActivePointer, planActivation, registryPaths} from '../../web-admin-v3/registry.js';

const app = 'demo';
const rel = id => ({
  app,
  id,
  ready: true,
  manifest: {
    schema: 'skrynia-native-static-v1',
    app,
    release: id,
    commit: 'c',
    repo: 'r',
    files: [{path: 'index.html', sha256: '00'.repeat(32)}],
  },
});
const a = 'a'.repeat(64);
const b = 'b'.repeat(64);

assert.deepEqual(normalizeActivePointer(null), {active: null, revision: 0});
assert.deepEqual(normalizeActivePointer(encodeActivePointer(null, 0)), {active: null, revision: 0});
assert.deepEqual(normalizeActivePointer(encodeActivePointer(a, 3)), {active: a, revision: 3});

let plan = planActivation(null, rel(a), null, 0);
assert.equal(plan.target, a);
assert.equal(plan.changed, true);
assert.deepEqual(plan.next, encodeActivePointer(a, 1));

plan = planActivation(plan.next, rel(b), a, 1);
assert.equal(plan.target, b);
assert.equal(plan.changed, true);
assert.deepEqual(plan.next, encodeActivePointer(b, 2));

plan = planActivation(plan.next, rel(a), b, 2);
assert.equal(plan.target, a);
assert.equal(plan.changed, true);
assert.deepEqual(plan.next, encodeActivePointer(a, 3));

assert.throws(() => planActivation(encodeActivePointer(b, 2), rel(a), b, 1), /revision changed/);
assert.throws(() => planActivation(encodeActivePointer(b, 2), rel(a), a, 2), /active release changed/);
assert.throws(() => planActivation(encodeActivePointer(a, 0), {...rel(b), ready: false}, a, 0), /not finalized/);
assert.throws(() => planActivation(encodeActivePointer(a, 0), {app, id: b, ready: true, manifest: {}}, a, 0), /Invalid release manifest/);

const paths = registryPaths(app);
assert.equal(paths.active, 'apps/demo/active.json');
assert.equal(paths.releaseManifest(a), `apps/demo/releases/${a}.json`);
assert.equal(paths.releaseReady(a), `apps/demo/releases/${a}.ready`);

assert.deepEqual(listAppIdsFromKeys([
  'apps/zeta/active.json',
  `apps/alpha/releases/${a}.json`,
  `apps/alpha/releases/${a}.ready`,
  'apps/bad-name/active.json',
  'apps/ok/releases/nothex.json',
  'other/alpha/active.json',
]), ['alpha', 'bad-name', 'zeta']);

console.log('v3 userland registry model tests passed');
