// Semantic capability adapter for the web-admin-v3 UI over the pinned native
// POST transport. It translates the four UI operations into the ordinary
// nonzero service's opaque AppGet/AppPut/AppCas wire operations and never
// handles the Program 0 bearer token: the trusted static origin's reverse
// proxy constructs the pinned P0 frame server-side.
//
// Authority is operator-provisioned session material (readCap/adminCap), never
// a static-bundle, URL or log value. The CAS precondition is the SHA-256 of the
// exact expected stored bytes, so activation is one bounded pointer write and
// is O(1) in both historical release count and release asset size.

import {createBinaryPostTransport} from './binary_transport.js';
import {assertAppName, assertReleaseId, normalizeManifest} from './capabilities.js';

const textEncoder = new TextEncoder();

const hexToBytes = hex => {
  if (typeof hex !== 'string' || !/^[0-9a-f]{64}$/.test(hex)) {
    throw Object.assign(Error('A 32-byte capability is required'), {code: 'invalid_input'});
  }
  const out = new Uint8Array(32);
  for (let i = 0; i < 32; i += 1) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  return out;
};

const putId = bytes => {
  const b = new Uint8Array(bytes);
  const out = new Uint8Array(2 + b.length);
  out[0] = b.length & 0xff;
  out[1] = (b.length >> 8) & 0xff;
  out.set(b, 2);
  return out;
};

const putBlob = bytes => {
  const b = new Uint8Array(bytes);
  const out = new Uint8Array(4 + b.length);
  new DataView(out.buffer).setUint32(0, b.length, true);
  out.set(b, 4);
  return out;
};

const concat = parts => {
  const total = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(total);
  let at = 0;
  for (const p of parts) { out.set(p, at); at += p.length; }
  return out;
};

const appGetOp = (iid, key) => concat([
  Uint8Array.from([0x01, 0x07]), putId(iid), putBlob(textEncoder.encode(key)),
]);
const appCasOp = (adminCap, key, expectedHash, value) => concat([
  Uint8Array.from([0x01, 0x06]), putId(adminCap), putBlob(textEncoder.encode(key)),
  Uint8Array.from([1]), expectedHash, putBlob(textEncoder.encode(value)),
]);

async function sha256(bytes) {
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return new Uint8Array(digest);
}

// Native response envelope: 0x01(version) || status || u32 LE length || body.
function decodeResponse(buf) {
  const b = new Uint8Array(buf);
  if (b.length < 6 || b[0] !== 1) {
    throw Object.assign(Error('Unrecognized native response framing'), {code: 'invalid_data'});
  }
  const len = new DataView(b.buffer, b.byteOffset, b.byteLength).getUint32(2, true);
  if (6 + len !== b.length) {
    throw Object.assign(Error('Malformed native response length'), {code: 'invalid_data'});
  }
  return {status: b[1], body: b.subarray(6)};
}

function parseJson(bytes, label) {
  try {
    return JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    throw Object.assign(Error('Invalid ' + label + ' payload from native service'), {code: 'invalid_data'});
  }
}

const keyApps = () => 'v3admin:apps';
const keyState = app => 'v3admin:app:' + app + ':state';
const keyVersions = app => 'v3admin:app:' + app + ':versions';

// Canonical, byte-exact state serialization shared with the provisioning seed.
// The CAS precondition hashes these exact bytes.
export const serializeState = state => JSON.stringify({
  active: state.active ?? null,
  pointerVersion: state.pointerVersion,
});

function assertState(value, app) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw Object.assign(Error('Invalid application state from native service'), {code: 'invalid_data'});
  }
  if (value.active !== null && (typeof value.active !== 'string' || !/^[0-9a-f]{64}$/.test(value.active))) {
    throw Object.assign(Error('Invalid active release pointer'), {code: 'invalid_data'});
  }
  if (!Number.isSafeInteger(value.pointerVersion) || value.pointerVersion < 0) {
    throw Object.assign(Error('Invalid pointer revision'), {code: 'invalid_data'});
  }
  return {app, active: value.active ?? null, pointerVersion: value.pointerVersion};
}

function assertVersionList(value, app) {
  if (!Array.isArray(value)) {
    throw Object.assign(Error('Invalid release history from native service'), {code: 'invalid_data'});
  }
  return value.map(v => {
    if (!v || typeof v !== 'object' || typeof v.id !== 'string' || !/^[0-9a-f]{64}$/.test(v.id)) {
      throw Object.assign(Error('Invalid release identifier in history'), {code: 'invalid_data'});
    }
    return {...v};
  });
}

function projectVersions(app, state, releases) {
  return releases.map(v => {
    let manifest = null;
    let error = null;
    try {
      if (v.ready !== true) throw Error('Release is not finalized');
      manifest = normalizeManifest(app, v.id, v.manifest);
    } catch (e) {
      error = e.message || 'Invalid release';
    }
    return {
      id: v.id,
      ready: v.ready === true,
      usable: error === null,
      error,
      commit: manifest?.commit || '',
      repo: manifest?.repo || '',
      files: manifest?.files ?? 0,
    };
  });
}

export function createNativeCapabilities(session, fetchImpl = globalThis.fetch) {
  if (!session || typeof session !== 'object') {
    throw Object.assign(Error('Operator session authority is required'), {code: 'unauthorized'});
  }
  if (typeof session.endpoint !== 'string' || !/^https?:\/\//.test(session.endpoint)) {
    throw Object.assign(Error('A trusted gateway endpoint is required'), {code: 'invalid_input'});
  }
  const iid = hexToBytes(session.readCap);
  const adminCap = hexToBytes(session.adminCap);
  const transport = createBinaryPostTransport(session.endpoint, fetchImpl);

  async function readKey(key, label) {
    const raw = await transport.post(appGetOp(iid, key));
    const decoded = decodeResponse(raw);
    if (decoded.status !== 0) {
      throw Object.assign(Error('Native read denied (' + label + ')'), {
        code: 'unauthorized', status: decoded.status,
      });
    }
    return decoded.body;
  }

  async function readApp(app) {
    assertAppName(app);
    const [stateBytes, versionBytes] = await Promise.all([
      readKey(keyState(app), 'state'),
      readKey(keyVersions(app), 'history'),
    ]);
    return {
      state: assertState(parseJson(stateBytes, 'application state'), app),
      releases: assertVersionList(parseJson(versionBytes, 'release history'), app),
    };
  }

  return {
    kind: 'native',
    async listApps() {
      const names = parseJson(await readKey(keyApps(), 'application list'), 'application list');
      if (!Array.isArray(names)) {
        throw Object.assign(Error('Invalid application list'), {code: 'invalid_data'});
      }
      return names.map(name => assertAppName(name)).sort();
    },
    async getAppState(app) {
      const {state, releases} = await readApp(app);
      return {
        name: app,
        active: state.active,
        pointerVersion: state.pointerVersion,
        versions: projectVersions(app, state, releases),
      };
    },
    async getReleaseManifest(app, releaseId) {
      assertAppName(app);
      assertReleaseId(releaseId);
      const {releases} = await readApp(app);
      const found = releases.find(v => v.id === releaseId);
      if (!found) throw Object.assign(Error('Release not found'), {code: 'not_found'});
      if (found.ready !== true) {
        throw Object.assign(Error('Release is not finalized'), {code: 'invalid_state'});
      }
      return normalizeManifest(app, releaseId, found.manifest);
    },
    async activateRelease(app, releaseId, opts = {}) {
      assertAppName(app);
      assertReleaseId(releaseId);
      if (!Number.isSafeInteger(opts.expectedPointerVersion) || opts.expectedPointerVersion < 0) {
        throw Object.assign(Error('A bounded pointer revision is required'), {code: 'invalid_input'});
      }
      // Fail closed locally before spending a CAS: never target an unsealed or
      // malformed historical release even if the UI is bypassed.
      const {releases} = await readApp(app);
      const target = releases.find(v => v.id === releaseId);
      if (!target) throw Object.assign(Error('Release not found'), {code: 'not_found'});
      if (target.ready !== true) {
        throw Object.assign(Error('Release is not finalized'), {code: 'invalid_state'});
      }
      try {
        normalizeManifest(app, releaseId, target.manifest);
      } catch {
        throw Object.assign(Error('Cannot activate release: invalid manifest'), {code: 'invalid_data'});
      }
      const expected = {
        active: opts.expectedActive ?? null,
        pointerVersion: opts.expectedPointerVersion,
      };
      const next = {active: releaseId, pointerVersion: opts.expectedPointerVersion + 1};
      const expectedHash = await sha256(textEncoder.encode(serializeState(expected)));
      const raw = await transport.post(
        appCasOp(adminCap, keyState(app), expectedHash, serializeState(next)));
      const decoded = decodeResponse(raw);
      if (decoded.status !== 0) {
        // The service reports a failed CAS precondition as a non-zero status.
        // Never replay: surface it as an explicit conflict for reconciliation.
        throw Object.assign(Error('Activation conflict: pointer changed concurrently'), {
          code: 'conflict', status: decoded.status,
        });
      }
      return {app, active: next.active, pointerVersion: next.pointerVersion};
    },
  };
}
