const NOT_FROZEN = 'Native Program 0 protocol not frozen; configure a tested capability adapter';

function fail(method) {
  return async () => {
    throw Object.assign(Error(NOT_FROZEN), {
      code: 'protocol_not_frozen',
      method,
    });
  };
}

export function createFailClosedCapabilities(reason = NOT_FROZEN) {
  const err = method => async () => {
    throw Object.assign(Error(reason), {code: 'capability_unavailable', method});
  };
  return {
    kind: 'fail-closed',
    listApps: err('listApps'),
    getAppState: err('getAppState'),
    getReleaseManifest: err('getReleaseManifest'),
    stageRelease: err('stageRelease'),
    sealRelease: err('sealRelease'),
    activateRelease: err('activateRelease'),
  };
}

export function asCapabilityError(error, fallback = 'Operation failed') {
  if (error && typeof error.message === 'string') return error;
  return Error(fallback);
}

export function assertAppName(name) {
  if (typeof name !== 'string' || !/^[a-z0-9][a-z0-9-]{0,63}$/.test(name)) {
    throw Object.assign(Error('Invalid application name'), {code: 'invalid_input'});
  }
  return name;
}

export function assertReleaseId(id) {
  if (typeof id !== 'string' || !/^[0-9a-f]{64}$/.test(id)) {
    throw Object.assign(Error('Invalid release identifier'), {code: 'invalid_input'});
  }
  return id;
}

export function normalizeManifest(app, id, manifest) {
  if (!manifest || manifest.schema !== 'skrynia-native-static-v1' || manifest.app !== app || manifest.release !== id) {
    throw Object.assign(Error('Invalid release manifest'), {code: 'invalid_data'});
  }
  if (!Array.isArray(manifest.files)) {
    throw Object.assign(Error('Invalid release manifest files'), {code: 'invalid_data'});
  }
  return {
    id,
    app,
    commit: typeof manifest.commit === 'string' ? manifest.commit : '',
    repo: typeof manifest.repo === 'string' ? manifest.repo : '',
    files: manifest.files.length,
    entries: manifest.files.map(f => ({...f})),
    raw: structuredClone(manifest),
  };
}

export function compareAndSwap(state, app, targetId, expectedActive, expectedPointerVersion) {
  assertAppName(app);
  assertReleaseId(targetId);
  const record = state.apps.get(app);
  if (!record) throw Object.assign(Error('Unknown application'), {code: 'not_found'});
  if (!Number.isSafeInteger(expectedPointerVersion) || expectedPointerVersion < 0) {
    throw Object.assign(Error('A bounded pointer version is required'), {code: 'invalid_input'});
  }
  if (record.pointerVersion !== expectedPointerVersion) {
    throw Object.assign(Error('Activation conflict: pointer version changed concurrently'),
      {code: 'conflict', currentVersion: record.pointerVersion,
        expectedVersion: expectedPointerVersion});
  }
  if (record.pointerVersion === Number.MAX_SAFE_INTEGER) {
    throw Object.assign(Error('Pointer version exhausted'), {code: 'limit'});
  }
  const release = record.releases.get(targetId);
  if (!release) throw Object.assign(Error('Selected release does not exist'), {code: 'not_found'});
  if (!release.ready) {
    throw Object.assign(Error('Selected release is not ready'), {code: 'invalid_state'});
  }
  try {
    normalizeManifest(app, targetId, release.manifest);
  } catch {
    throw Object.assign(Error('Cannot activate release: invalid manifest'), {code: 'invalid_data'});
  }
  if (expectedActive !== undefined && expectedActive !== null) {
    if (typeof expectedActive !== 'string' || !/^[0-9a-f]{64}$/.test(expectedActive)) {
      throw Object.assign(Error('Invalid expected active release'), {code: 'invalid_input'});
    }
    if (record.active !== expectedActive) {
      throw Object.assign(
        Error('Activation conflict: active release changed concurrently'),
        {code: 'conflict', current: record.active, expected: expectedActive});
    }
  } else if (record.active !== null) {
    throw Object.assign(
      Error('Activation conflict: active release already exists'),
      {code: 'conflict', current: record.active, expected: null});
  }
  // O(1) pointer swap only; no release assets or app data copied/migrated.
  record.active = targetId;
  record.pointerVersion = (record.pointerVersion || 0) + 1;
  return {app, active: targetId, pointerVersion: record.pointerVersion};
}

export function createFakeCapabilities(seed = {}) {
  const apps = new Map();
  for (const [name, spec] of Object.entries(seed.apps || {})) {
    assertAppName(name);
    const releases = new Map();
    for (const rel of spec.releases || []) {
      assertReleaseId(rel.id);
      releases.set(rel.id, {
        id: rel.id,
        ready: rel.ready !== false,
        manifest: rel.manifest ? structuredClone(rel.manifest) : null,
      });
    }
    apps.set(name, {
      active: spec.active ?? null,
      releases,
      pointerVersion: (() => {
        const version = spec.pointerVersion ?? 0;
        if (!Number.isSafeInteger(version) || version < 0) {
          throw Object.assign(Error('Invalid pointer version'), {code: 'invalid_data'});
        }
        return version;
      })(),
    });
  }
  const state = {apps};

  return {
    kind: 'fake',
    state,
    async listApps() {
      return [...apps.keys()].sort();
    },
    async getAppState(name) {
      assertAppName(name);
      const record = apps.get(name);
      if (!record) throw Object.assign(Error('Unknown application'), {code: 'not_found'});
      const versions = [];
      for (const [id, release] of [...record.releases.entries()].sort(([a], [b]) => a.localeCompare(b))) {
        let manifest = null;
        let error = null;
        try {
          if (!release.ready) throw Error('Release is not finalized');
          manifest = normalizeManifest(name, id, release.manifest);
        } catch (e) {
          error = e.message || 'Invalid release';
        }
        versions.push({
          id,
          ready: release.ready === true,
          usable: error === null,
          error,
          commit: manifest?.commit || '',
          repo: manifest?.repo || '',
          files: manifest?.files ?? 0,
        });
      }
      return {
        name,
        active: record.active,
        pointerVersion: record.pointerVersion || 0,
        versions,
      };
    },
    async getReleaseManifest(name, id) {
      assertAppName(name);
      assertReleaseId(id);
      const record = apps.get(name);
      if (!record) throw Object.assign(Error('Unknown application'), {code: 'not_found'});
      const release = record.releases.get(id);
      if (!release) throw Object.assign(Error('Release not found'), {code: 'not_found'});
      if (!release.ready) {
        throw Object.assign(Error('Release is not finalized'), {code: 'invalid_state'});
      }
      return normalizeManifest(name, id, release.manifest);
    },
    async activateRelease(name, id, opts = {}) {
      return compareAndSwap(state, name, id, opts.expectedActive, opts.expectedPointerVersion);
    },
  };
}
