// Semantic capability adapter for the web-admin-v3 UI over the pinned native
// POST transport. It speaks the dedicated admin ABI of the ordinary nonzero
// WASM service: STAGE=9, SEAL=10, ACTIVATE=11, GET_ACTIVE=12,
// LIST_VERSIONS=13 and LIST_PROGRAMS=14 (the app-ID listing operation; there
// is no app listing among opcodes 9-13). AppPut/AppGet (6/7) are never sent:
// the UI no longer simulates releases with client-side appdata JSON.
//
// The browser never handles the Program 0 bearer token: the trusted static
// origin's reverse proxy constructs the pinned P0 frame server-side. Authority
// is operator-provisioned session material (invoke/read ID, admin capability),
// never a static-bundle, URL or log value. Activation is one fixed-size u64
// revision CAS enforced by the compiled service, O(1) in version count and
// release size, and a lost/malformed response is never replayed.

import {createBinaryPostTransport} from './binary_transport.js';
import {assertAppName, assertReleaseId, normalizeManifest} from './capabilities.js';

const textEncoder = new TextEncoder();
const PAGE_LIMIT = 256; // compiled service rejects limit=0 and limit>256
const OP = {
  STAGE: 9, SEAL: 10, ACTIVATE: 11, GET_ACTIVE: 12, LIST_VERSIONS: 13, LIST_PROGRAMS: 14,
};
// Native service error codes (response status 1, body[0]).
const ERR = {BADREQ: 1, IO: 7, NOTFOUND: 8, UNAUTH: 9, CONFLICT: 10};

const invalidInput = message => Object.assign(Error(message), {code: 'invalid_input'});
const invalidData = message => Object.assign(Error(message), {code: 'invalid_data'});

const hexToBytes = hex => {
  if (typeof hex !== 'string' || !/^[0-9a-f]{64}$/.test(hex)) {
    throw invalidInput('A 32-byte capability or identifier is required');
  }
  const out = new Uint8Array(32);
  for (let i = 0; i < 32; i += 1) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  return out;
};

const bytesToHex = bytes => {
  let out = '';
  for (const byte of bytes) out += byte.toString(16).padStart(2, '0');
  return out;
};

const putId = bytes => {
  const b = new Uint8Array(bytes);
  const out = new Uint8Array(2 + b.length);
  const view = new DataView(out.buffer);
  view.setUint16(0, b.length, true);
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

const pageBytes = (after, limit) => {
  const l = new Uint8Array(2);
  new DataView(l.buffer).setUint16(0, limit, true);
  return after ? concat([Uint8Array.from([1]), after, l]) : concat([Uint8Array.from([0]), l]);
};

const stageOp = (admin, code) =>
  concat([Uint8Array.from([1, OP.STAGE]), putId(admin), putBlob(code)]);
const sealOp = (admin, hash) =>
  concat([Uint8Array.from([1, OP.SEAL]), putId(admin), hash]);
const activateOp = (admin, revision, target) => {
  const r = new Uint8Array(8);
  new DataView(r.buffer).setBigUint64(0, BigInt(revision), true);
  return concat([Uint8Array.from([1, OP.ACTIVATE]), putId(admin), r, target]);
};
const getActiveOp = iid => concat([Uint8Array.from([1, OP.GET_ACTIVE]), putId(iid)]);
const listVersionsOp = (iid, after) =>
  concat([Uint8Array.from([1, OP.LIST_VERSIONS]), putId(iid), pageBytes(after, PAGE_LIMIT)]);
const listProgramsOp = after =>
  concat([Uint8Array.from([1, OP.LIST_PROGRAMS]), pageBytes(after, PAGE_LIMIT)]);

async function sha256(bytes) {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
}

// Native response envelope: 0x01(version) || status || u32 LE length || body.
function decodeResponse(buf) {
  const b = new Uint8Array(buf);
  if (b.length < 6 || b[0] !== 1) {
    throw invalidData('Unrecognized native response framing');
  }
  const len = new DataView(b.buffer, b.byteOffset, b.byteLength).getUint32(2, true);
  if (6 + len !== b.length) {
    throw invalidData('Malformed native response length');
  }
  return {status: b[1], body: b.subarray(6)};
}

function parseActive(bytes, label) {
  if (bytes.length !== 42 || bytes[0] !== 3 || bytes[1] > 1) {
    throw invalidData('Malformed active record from native service (' + label + ')');
  }
  const revision = Number(new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
    .getBigUint64(34, true));
  if (!Number.isSafeInteger(revision)) {
    throw invalidData('Active pointer revision exceeds the browser-safe range');
  }
  return {
    live: bytes[1] === 1,
    hash: bytesToHex(bytes.subarray(2, 34)),
    revision,
  };
}

function parsePage(bytes, label) {
  if (bytes.length < 2) throw invalidData('Malformed native page (' + label + ')');
  const count = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint16(0, true);
  if (count > PAGE_LIMIT) {
    throw invalidData('Native page exceeds ' + PAGE_LIMIT + ' entries (' + label + ')');
  }
  if (bytes.length !== 2 + count * 32) {
    throw invalidData('Malformed native page length (' + label + ')');
  }
  const hashes = [];
  let previous = null;
  for (let i = 0; i < count; i += 1) {
    const hex = bytesToHex(bytes.subarray(2 + i * 32, 2 + (i + 1) * 32));
    // The service indexes 32-byte keys in strictly ascending order. Duplicates
    // and unsorted pages are malformed service data, never UI state to repair.
    if (previous !== null && hex <= previous) {
      throw invalidData('Native page hashes are not strictly ascending (' + label + ')');
    }
    hashes.push(hex);
    previous = hex;
  }
  return {hashes, last: count ? bytes.subarray(2 + (count - 1) * 32, 2 + count * 32) : null};
}

function serviceError(status, body, label, unreadyCode = 'not_found') {
  const nativeError = body.length ? body[0] : null;
  let code = 'invalid_data';
  if (nativeError === ERR.NOTFOUND) code = unreadyCode;
  else if (nativeError === ERR.UNAUTH) code = 'unauthorized';
  else if (nativeError === ERR.CONFLICT) code = 'conflict';
  else if (nativeError === ERR.IO) code = 'io';
  return Object.assign(Error(label + ' rejected by native service (err ' + nativeError + ')'), {
    code, status, nativeError,
  });
}

function gatewayDenied(error) {
  return error && error.code === 'rejected' && (error.status === 401 || error.status === 403);
}

// A mutation that was submitted but whose response cannot be trusted has an
// unknown outcome: the u64 CAS may or may not have committed. Report that state
// explicitly so callers never auto-retry or silently replay the write.
function uncertainActivation(app, target, expectedPointerVersion, message, cause) {
  const error = Error(message);
  error.code = 'uncertain';
  error.app = app;
  error.target = target;
  error.expectedPointerVersion = expectedPointerVersion;
  if (cause !== undefined) error.cause = cause;
  return error;
}

// The compiled service stores sealed content hashes, not static file manifests.
// Expose an identity-only manifest so "Inspect manifest" stays honest: no file
// list, commit or repository is fabricated from a hash.
function identityManifest(app, releaseId) {
  return normalizeManifest(app, releaseId, {
    schema: 'skrynia-native-static-v1',
    app,
    release: releaseId,
    commit: '',
    repo: '',
    files: [],
    origin: 'native-p1-ready-hash',
  });
}

export function createNativeCapabilities(session, fetchImpl = globalThis.fetch) {
  if (!session || typeof session !== 'object') {
    throw Object.assign(Error('Operator session authority is required'), {code: 'unauthorized'});
  }
  if (typeof session.endpoint !== 'string' || !/^https?:\/\//.test(session.endpoint)) {
    throw invalidInput('A trusted gateway endpoint is required');
  }
  const readCap = hexToBytes(session.readCap);
  const adminCap = hexToBytes(session.adminCap);
  const transport = createBinaryPostTransport(session.endpoint, fetchImpl);
  const appAdminCaps = session.adminCaps && typeof session.adminCaps === 'object'
    ? session.adminCaps : null;

  const invokeIdFor = app => {
    assertAppName(app);
    return app === bytesToHex(readCap) ? readCap : hexToBytes(app);
  };

  async function adminCapFor(app) {
    assertAppName(app);
    if (appAdminCaps && typeof appAdminCaps[app] === 'string') {
      return hexToBytes(appAdminCaps[app]);
    }
    if (app === bytesToHex(await sha256(adminCap))) return adminCap;
    throw Object.assign(Error('No operator admin authority provisioned for application ' + app), {
      code: 'unauthorized', app,
    });
  }

  async function sendRead(frame, label) {
    let raw;
    try {
      raw = await transport.post(frame);
    } catch (error) {
      if (gatewayDenied(error)) {
        throw Object.assign(Error(label + ' denied by gateway'), {
          code: 'unauthorized', status: error.status, cause: error,
        });
      }
      throw Object.assign(Error(label + ' could not be read from the native service'), {
        code: 'transport_unknown', cause: error,
      });
    }
    let decoded;
    try {
      decoded = decodeResponse(raw);
    } catch (error) {
      throw invalidData('Malformed native response (' + label + ')');
    }
    if (decoded.status !== 0) throw serviceError(decoded.status, decoded.body, label);
    return decoded.body;
  }

  async function sendMutation(frame, label, mutation) {
    let raw;
    try {
      raw = await transport.post(frame);
    } catch (error) {
      if (gatewayDenied(error)) {
        throw Object.assign(Error(label + ' denied by gateway'), {
          code: 'unauthorized', status: error.status, cause: error,
        });
      }
      throw uncertainActivation(mutation.app, mutation.target, mutation.expectedPointerVersion,
        label + ' outcome unknown: response lost or rejected; refresh to reconcile', error);
    }
    let decoded;
    try {
      decoded = decodeResponse(raw);
    } catch (error) {
      throw uncertainActivation(mutation.app, mutation.target, mutation.expectedPointerVersion,
        label + ' outcome unknown: malformed service response; refresh to reconcile', error);
    }
    // A service status is definitive: the compiled service reports a failed
    // precondition/CAS without writing, so this is never an unknown outcome.
    if (decoded.status !== 0) {
      throw serviceError(decoded.status, decoded.body, label,
        label === 'Activation' ? 'invalid_state' : 'not_found');
    }
    return decoded.body;
  }

  async function listAll(iid, label) {
    const seen = new Set();
    const out = [];
    let after = null;      // byte cursor for the next request
    let afterHex = null;   // hex cursor for strict ordering checks
    for (let guard = 0; guard < 4096; guard += 1) {
      const body = await sendRead(
        iid ? listVersionsOp(iid, after) : listProgramsOp(after), label);
      const page = parsePage(body, label);
      if (afterHex !== null && page.hashes.length && page.hashes[0] <= afterHex) {
        throw invalidData(label + ' pagination cursor moved backwards or repeated');
      }
      out.push(...page.hashes);
      if (page.hashes.length < PAGE_LIMIT || !page.last) return out;
      const cursor = bytesToHex(page.last);
      if (seen.has(cursor) || cursor <= afterHex) {
        throw invalidData(label + ' pagination cursor did not advance');
      }
      seen.add(cursor);
      after = page.last;
      afterHex = cursor;
    }
    // Never return a truncated list when the guard is exhausted.
    throw invalidData(label + ' pagination guard exhausted; refusing partial list');
  }

  return {
    kind: 'native',
    async listApps() {
      const apps = await listAll(null, 'application list');
      for (let i = 1; i < apps.length; i += 1) {
        if (apps[i - 1] >= apps[i]) {
          throw invalidData('application list is not strictly ascending and unique');
        }
      }
      return apps;
    },
    async getAppState(app) {
      const active = parseActive(
        await sendRead(getActiveOp(invokeIdFor(app)), 'active pointer'), 'active pointer');
      const ready = await listAll(invokeIdFor(app), 'ready versions');
      return {
        name: app,
        active: active.live ? active.hash : null,
        pointerVersion: active.revision,
        versions: ready.map(id => ({
          id, ready: true, usable: true, error: null,
          commit: '', repo: '', files: 0,
        })),
      };
    },
    async getReleaseManifest(app, releaseId) {
      assertAppName(app);
      assertReleaseId(releaseId);
      if (!(await listAll(invokeIdFor(app), 'ready versions')).includes(releaseId)) {
        throw Object.assign(Error('Release not found among ready versions'), {code: 'not_found'});
      }
      return identityManifest(app, releaseId);
    },
    async stageRelease(app, code) {
      assertAppName(app);
      if (!(code instanceof Uint8Array) || code.length === 0 || code.length > 8 * 1024 * 1024) {
        throw invalidInput('Stage requires non-empty code within the service limit');
      }
      const admin = await adminCapFor(app);
      const body = await sendMutation(stageOp(admin, code), 'Stage',
        {app, target: null, expectedPointerVersion: undefined});
      if (body.length !== 32) {
        throw uncertainActivation(app, null, undefined,
          'Stage outcome unknown: unexpected service response; refresh to reconcile');
      }
      return bytesToHex(body);
    },
    async sealRelease(app, releaseId) {
      assertAppName(app);
      assertReleaseId(releaseId);
      const admin = await adminCapFor(app);
      const body = await sendMutation(sealOp(admin, hexToBytes(releaseId)), 'Seal',
        {app, target: releaseId, expectedPointerVersion: undefined});
      if (body.length !== 0) {
        throw uncertainActivation(app, releaseId, undefined,
          'Seal outcome unknown: unexpected service response; refresh to reconcile');
      }
      return {app, release: releaseId, sealed: true};
    },
    async activateRelease(app, releaseId, opts = {}) {
      assertAppName(app);
      assertReleaseId(releaseId);
      const expected = opts.expectedPointerVersion;
      if (!Number.isSafeInteger(expected) || expected < 1) {
        throw invalidInput('A bounded non-zero pointer revision is required');
      }
      const admin = await adminCapFor(app);
      const iid = invokeIdFor(app);
      // Local fail-closed preflight: never spend the single CAS on a plan that
      // is already known to be stale or to target an unsealed release.
      const current = parseActive(
        await sendRead(getActiveOp(iid), 'active pointer'), 'active pointer');
      if (opts.expectedActive != null) {
        assertReleaseId(opts.expectedActive);
        if (!current.live || current.hash !== opts.expectedActive) {
          throw Object.assign(Error('Activation conflict: active release changed'), {
            code: 'conflict', current: current.hash, expected: opts.expectedActive,
          });
        }
      }
      if (!current.live || current.revision !== expected) {
        throw Object.assign(Error('Activation conflict: pointer revision changed'), {
          code: 'conflict', current: current.revision, expected,
        });
      }
      if (!(await listAll(iid, 'ready versions')).includes(releaseId)) {
        throw Object.assign(Error('Release is not sealed and ready'), {code: 'invalid_state'});
      }
      const body = await sendMutation(
        activateOp(admin, expected, hexToBytes(releaseId)), 'Activation',
        {app, target: releaseId, expectedPointerVersion: expected});
      let next;
      try {
        next = parseActive(body, 'activated pointer');
      } catch (error) {
        throw uncertainActivation(app, releaseId, expected,
          'Activation outcome unknown: malformed service response; refresh to reconcile', error);
      }
      if (!next.live || next.hash !== releaseId || next.revision !== expected + 1) {
        throw uncertainActivation(app, releaseId, expected,
          'Activation outcome unknown: unexpected service response; refresh to reconcile');
      }
      return {app, active: next.hash, pointerVersion: next.revision};
    },
  };
}
