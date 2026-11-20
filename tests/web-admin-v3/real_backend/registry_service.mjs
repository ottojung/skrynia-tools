import {mkdir, readFile, writeFile, rename, readdir, stat} from 'node:fs/promises';
import {join} from 'node:path';
import {randomBytes} from 'node:crypto';

const APP_RE = /^[a-z0-9][a-z0-9-]{0,63}$/;
const REL_RE = /^[0-9a-f]{64}$/;

function assertApp(name) {
  if (typeof name !== 'string' || !APP_RE.test(name)) {
    throw Object.assign(Error('Invalid application name'), {code: 'invalid_input'});
  }
  return name;
}
function assertRelease(id) {
  if (typeof id !== 'string' || !REL_RE.test(id)) {
    throw Object.assign(Error('Invalid release identifier'), {code: 'invalid_input'});
  }
  return id;
}
function normalizeManifest(app, id, manifest) {
  if (!manifest || manifest.schema !== 'skrynia-native-static-v1' || manifest.app !== app || manifest.release !== id) {
    throw Object.assign(Error('Invalid release manifest'), {code: 'invalid_data'});
  }
  if (!Array.isArray(manifest.files)) {
    throw Object.assign(Error('Invalid release manifest files'), {code: 'invalid_data'});
  }
  return {
    id, app,
    commit: typeof manifest.commit === 'string' ? manifest.commit : '',
    repo: typeof manifest.repo === 'string' ? manifest.repo : '',
    files: manifest.files.length,
    entries: manifest.files.map(f => ({...f})),
    raw: structuredClone(manifest),
  };
}
async function readJson(path) {
  return JSON.parse(await readFile(path, 'utf8'));
}
async function writeJsonAtomic(path, value) {
  const tmp = path + '.tmp.' + randomBytes(8).toString('hex');
  await writeFile(tmp, JSON.stringify(value) + '\n');
  await rename(tmp, path);
}

export class FileRegistryService {
  constructor(rootDir) {
    this.root = rootDir;
  }
  appDir(app) { return join(this.root, 'apps', assertApp(app)); }
  activePath(app) { return join(this.appDir(app), 'active.json'); }
  releasesDir(app) { return join(this.appDir(app), 'releases'); }
  releasePath(app, id) { return join(this.releasesDir(app), assertRelease(id) + '.json'); }
  readyPath(app, id) { return join(this.releasesDir(app), assertRelease(id) + '.ready'); }

  async init() {
    await mkdir(join(this.root, 'apps'), {recursive: true});
  }

  async seedApp(name, {releases = [], active = null, revision = 0} = {}) {
    assertApp(name);
    const appDir = this.appDir(name);
    await mkdir(join(appDir, 'releases'), {recursive: true});
    for (const rel of releases) {
      assertRelease(rel.id);
      const manifest = rel.manifest ? structuredClone(rel.manifest) : null;
      await writeJsonAtomic(this.releasePath(name, rel.id), {
        id: rel.id, ready: rel.ready !== false, manifest,
      });
      if (rel.ready !== false) {
        await writeFile(this.readyPath(name, rel.id), 'ready\n');
      }
    }
    if (active !== null || revision !== 0) {
      if (active !== null) assertRelease(active);
      if (!Number.isSafeInteger(revision) || revision < 0) throw Error('Invalid revision');
      await writeJsonAtomic(this.activePath(name), {
        schema: 'skrynia-userland-pointer-v1', active, revision,
      });
    }
  }

  async listApps() {
    const appsDir = join(this.root, 'apps');
    let entries;
    try { entries = await readdir(appsDir, {withFileTypes: true}); }
    catch (e) { if (e && e.code === 'ENOENT') return []; throw e; }
    return entries.filter(e => e.isDirectory() && APP_RE.test(e.name)).map(e => e.name).sort();
  }

  async readPointer(app) {
    try {
      const ptr = await readJson(this.activePath(app));
      if (!ptr || ptr.schema !== 'skrynia-userland-pointer-v1') {
        throw Object.assign(Error('Invalid active pointer'), {code: 'invalid_data'});
      }
      if (ptr.active !== null) assertRelease(ptr.active);
      if (!Number.isSafeInteger(ptr.revision) || ptr.revision < 0) {
        throw Object.assign(Error('Invalid pointer revision'), {code: 'invalid_data'});
      }
      return {active: ptr.active, revision: ptr.revision};
    } catch (e) {
      if (e && e.code === 'ENOENT') return {active: null, revision: 0};
      throw e;
    }
  }

  async readRelease(app, id) {
    assertRelease(id);
    let rec;
    try { rec = await readJson(this.releasePath(app, id)); }
    catch (e) { if (e && e.code === 'ENOENT') return null; throw e; }
    let ready = rec && rec.ready === true;
    try { await stat(this.readyPath(app, id)); } catch { ready = false; }
    return {id, ready, manifest: rec ? rec.manifest : null};
  }

  async listReleaseIds(app) {
    let entries;
    try { entries = await readdir(this.releasesDir(app)); }
    catch (e) { if (e && e.code === 'ENOENT') return []; throw e; }
    const ids = new Set();
    for (const name of entries) {
      const m = name.match(/^([0-9a-f]{64})\.(json|ready)$/);
      if (m) ids.add(m[1]);
    }
    return [...ids].sort();
  }

  async getAppState(app) {
    assertApp(app);
    // verify app exists
    try { await stat(this.appDir(app)); }
    catch (e) { if (e && e.code === 'ENOENT') throw Object.assign(Error('Unknown application'), {code: 'not_found'}); throw e; }
    const pointer = await this.readPointer(app);
    const ids = await this.listReleaseIds(app);
    const versions = [];
    for (const id of ids) {
      const release = await this.readRelease(app, id);
      let manifest = null;
      let error = null;
      try {
        if (!release || !release.ready) throw Error('Release is not finalized');
        manifest = normalizeManifest(app, id, release.manifest);
      } catch (e) {
        error = e && e.message ? e.message : 'Invalid release';
      }
      versions.push({
        id, ready: !!(release && release.ready), usable: error === null,
        error, commit: manifest?.commit || '', repo: manifest?.repo || '',
        files: manifest?.files ?? 0,
      });
    }
    versions.sort((a,b) => a.id.localeCompare(b.id));
    return {name: app, active: pointer.active, pointerVersion: pointer.revision, versions};
  }

  async getReleaseManifest(app, id) {
    assertApp(app); assertRelease(id);
    const release = await this.readRelease(app, id);
    if (!release) throw Object.assign(Error('Release not found'), {code: 'not_found'});
    if (!release.ready) throw Object.assign(Error('Release is not finalized'), {code: 'invalid_state'});
    return normalizeManifest(app, id, release.manifest);
  }

  async activateRelease(app, id, {expectedActive, expectedPointerVersion} = {}) {
    assertApp(app); assertRelease(id);
    // constant-work pointer CAS: read pointer, validate, atomic replace pointer file only
    const pointer = await this.readPointer(app);
    if (!Number.isSafeInteger(expectedPointerVersion) || expectedPointerVersion < 0) {
      throw Object.assign(Error('A bounded pointer version is required'), {code: 'invalid_input'});
    }
    if (pointer.revision !== expectedPointerVersion) {
      throw Object.assign(Error('Activation conflict: pointer version changed concurrently'),
        {code: 'conflict', currentVersion: pointer.revision, expectedVersion: expectedPointerVersion});
    }
    if (expectedActive !== undefined && expectedActive !== null) {
      if (typeof expectedActive !== 'string' || !REL_RE.test(expectedActive)) {
        throw Object.assign(Error('Invalid expected active release'), {code: 'invalid_input'});
      }
      if (pointer.active !== expectedActive) {
        throw Object.assign(Error('Activation conflict: active release changed concurrently'),
          {code: 'conflict', current: pointer.active, expected: expectedActive});
      }
    } else if (pointer.active !== null) {
      throw Object.assign(Error('Activation conflict: active release already exists'),
        {code: 'conflict', current: pointer.active, expected: null});
    }
    const release = await this.readRelease(app, id);
    if (!release) throw Object.assign(Error('Selected release does not exist'), {code: 'not_found'});
    if (!release.ready) throw Object.assign(Error('Selected release is not ready'), {code: 'invalid_state'});
    try { normalizeManifest(app, id, release.manifest); }
    catch { throw Object.assign(Error('Cannot activate release: invalid manifest'), {code: 'invalid_data'}); }
    if (pointer.revision === Number.MAX_SAFE_INTEGER) {
      throw Object.assign(Error('Pointer version exhausted'), {code: 'limit'});
    }
    const next = {schema: 'skrynia-userland-pointer-v1', active: id, revision: pointer.revision + 1};
    await writeJsonAtomic(this.activePath(app), next);
    return {app, active: id, pointerVersion: next.revision};
  }

  async handleRequest(req) {
    if (!req || typeof req.op !== 'string') throw Object.assign(Error('Invalid request'), {code: 'invalid_input'});
    switch (req.op) {
      case 'listApps': return await this.listApps();
      case 'getAppState': return await this.getAppState(req.app);
      case 'getReleaseManifest': return await this.getReleaseManifest(req.app, req.releaseId);
      case 'activateRelease': return await this.activateRelease(req.app, req.releaseId, req.opts || {});
      default: throw Object.assign(Error('Unsupported operation'), {code: 'unsupported'});
    }
  }
}
