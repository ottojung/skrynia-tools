import assert from 'node:assert/strict';
import {PAGE_SIZE, isUncertainActivationError, paginateVersions} from '../../web-admin-v3/apps_ui.js';
import {createFakeCapabilities} from '../../web-admin-v3/capabilities.js';
import {loadAllAppStates} from '../../web-admin-v3/apps.js';

function manifest(app, id) {
  return {
    schema: 'skrynia-native-static-v1',
    app,
    release: id,
    commit: 'c-' + id.slice(0, 8),
    repo: 'ottojung/example',
    files: [{path: 'index.html', bytes: 10, sha256: 'f'.repeat(64)}],
  };
}

{
  assert.equal(isUncertainActivationError(Object.assign(Error('x'), {code: 'transport_unknown'})), true);
  assert.equal(isUncertainActivationError(Object.assign(Error('x'), {code: 'uncertain'})), true);
  assert.equal(isUncertainActivationError(Object.assign(Error('x'), {code: 'lost_ack'})), true);
  assert.equal(isUncertainActivationError(Error('Program 0 request failed; outcome unknown')), true);
  assert.equal(isUncertainActivationError(Object.assign(Error('conflict'), {code: 'conflict'})), false);
  assert.equal(isUncertainActivationError(Error('Invalid release manifest')), false);
  assert.equal(isUncertainActivationError(null), false);
}

{
  const versions = Array.from({length: 1000}, (_, i) => ({id: i.toString(16).padStart(64, '0')}));
  const p1 = paginateVersions(versions, 1);
  assert.equal(PAGE_SIZE, 50);
  assert.equal(p1.total, 1000);
  assert.equal(p1.totalPages, 20);
  assert.equal(p1.page, 1);
  assert.equal(p1.items.length, 50);
  assert.equal(p1.items[0].id, versions[0].id);
  assert.equal(p1.items[49].id, versions[49].id);

  const p2 = paginateVersions(versions, 2);
  assert.equal(p2.items[0].id, versions[50].id);
  const p20 = paginateVersions(versions, 20);
  assert.equal(p20.page, 20);
  assert.equal(p20.items.length, 50);
  assert.equal(p20.items[49].id, versions[999].id);

  const seen = new Set();
  for (let page = 1; page <= p1.totalPages; page++) {
    const p = paginateVersions(versions, page);
    for (const v of p.items) {
      assert.equal(seen.has(v.id), false, 'version must appear exactly once across pages');
      seen.add(v.id);
    }
  }
  assert.equal(seen.size, 1000, 'all versions must be reachable via navigation');

  const under = paginateVersions(versions, 0);
  assert.equal(under.page, 1);
  const over = paginateVersions(versions, 999);
  assert.equal(over.page, 20);
  const empty = paginateVersions([], 1);
  assert.equal(empty.totalPages, 1);
  assert.equal(empty.items.length, 0);
}

{
  const releases = Array.from({length: 1000}, (_, i) => {
    const id = (i + 1).toString(16).padStart(64, '0');
    return {id, ready: true, manifest: manifest('bigapp', id)};
  });
  const active = releases[499].id;
  const caps = createFakeCapabilities({
    apps: {
      bigapp: {active, pointerVersion: 7, releases},
    },
  });
  const states = await loadAllAppStates(caps);
  assert.equal(states.length, 1);
  assert.equal(states[0].versions.length, 1000);
  assert.equal(states[0].active, active);
  assert.ok(states[0].versions.every(v => v.usable && v.ready));
  // Every historical version is present and selectable in principle.
  const ids = new Set(states[0].versions.map(v => v.id));
  assert.equal(ids.size, 1000);
  for (const r of releases) assert.equal(ids.has(r.id), true);
}

console.log('v3 pagination and uncertain-activation classification tests passed');
