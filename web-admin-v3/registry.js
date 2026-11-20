import {assertAppName, assertReleaseId, normalizeManifest} from './capabilities.js';

const POINTER_SCHEMA = 'skrynia-userland-pointer-v1';
const APP_NAME_RE = /^[a-z0-9][a-z0-9-]{0,63}$/;

export function assertRegistryAppName(name) {
  assertAppName(name);
  if (!APP_NAME_RE.test(name)) {
    throw Object.assign(Error('Invalid application name'), {code: 'invalid_input'});
  }
  return name;
}

export function normalizeRegistryRelease(app, id, release) {
  assertRegistryAppName(app);
  assertReleaseId(id);
  if (!release || typeof release !== 'object') {
    throw Object.assign(Error('Invalid release record'), {code: 'invalid_data'});
  }
  const ready = release.ready === true;
  let manifest = null;
  let error = null;
  if (ready) {
    try {
      manifest = normalizeManifest(app, id, release.manifest);
    } catch (e) {
      error = e && e.message ? e.message : 'Invalid release manifest';
    }
  } else {
    error = 'Release is not finalized';
  }
  return {
    id,
    ready,
    usable: ready && error === null,
    error,
    commit: manifest?.commit || '',
    repo: manifest?.repo || '',
    files: manifest?.files ?? 0,
    manifest: manifest ? manifest.raw : null,
  };
}

export function normalizeActivePointer(value) {
  if (value == null) return {active: null, revision: 0};
  if (!value || typeof value !== 'object' || value.schema !== POINTER_SCHEMA) {
    throw Object.assign(Error('Invalid active pointer'), {code: 'invalid_data'});
  }
  if (value.active !== null) assertReleaseId(value.active);
  if (!Number.isSafeInteger(value.revision) || value.revision < 0) {
    throw Object.assign(Error('Invalid pointer revision'), {code: 'invalid_data'});
  }
  return {active: value.active, revision: value.revision};
}

export function encodeActivePointer(active, revision) {
  if (active !== null) assertReleaseId(active);
  if (!Number.isSafeInteger(revision) || revision < 0) {
    throw Object.assign(Error('Invalid pointer revision'), {code: 'invalid_input'});
  }
  return {schema: POINTER_SCHEMA, active, revision};
}

export function planActivation(currentPointer, release, expectedActive, expectedRevision) {
  const pointer = normalizeActivePointer(currentPointer);
  const rel = normalizeRegistryRelease(release.app || release.name, release.id, release);
  if (expectedActive !== pointer.active) {
    throw Object.assign(Error('Activation conflict: active release changed concurrently'),
      {code: 'conflict', current: pointer.active, expected: expectedActive});
  }
  if (expectedRevision !== pointer.revision) {
    throw Object.assign(Error('Activation conflict: pointer revision changed concurrently'),
      {code: 'conflict', currentRevision: pointer.revision, expectedRevision});
  }
  if (!rel.usable) {
    throw Object.assign(Error('Cannot activate release: ' + (rel.error || 'unavailable')),
      {code: 'invalid_state'});
  }
  if (pointer.revision === Number.MAX_SAFE_INTEGER) {
    throw Object.assign(Error('Pointer revision exhausted'), {code: 'limit'});
  }
  const next = encodeActivePointer(rel.id, pointer.revision + 1);
  return {
    next,
    changed: pointer.active !== rel.id,
    current: pointer,
    target: rel.id,
  };
}

export function registryPaths(app) {
  assertRegistryAppName(app);
  return {
    active: `apps/${app}/active.json`,
    releasesDir: `apps/${app}/releases`,
    releaseManifest: id => `apps/${app}/releases/${assertReleaseId(id)}.json`,
    releaseReady: id => `apps/${app}/releases/${assertReleaseId(id)}.ready`,
  };
}

export function listAppIdsFromKeys(keys) {
  if (!Array.isArray(keys)) throw Error('Invalid key listing');
  const names = new Set();
  for (const key of keys) {
    if (typeof key !== 'string') continue;
    const m = key.match(/^apps\/([a-z0-9][a-z0-9-]{0,63})\/(?:active\.json|releases\/[0-9a-f]{64}\.(?:json|ready))$/);
    if (m) names.add(m[1]);
  }
  return [...names].sort();
}
