#!/usr/bin/env node
// One-time native-v3 trusted-host admin bootstrap. No subcommands, no CLI.
// Publishes immutable admin assets through the operator's already-deployed
// publication root/provider helper and prints link + password only after every
// served file has been independently SHA-256 verified.
import {createHash, randomBytes} from 'node:crypto';
import {constants, statSync} from 'node:fs';
import {lstat, mkdir, open, readFile, readdir, writeFile} from 'node:fs/promises';
import {join, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {encryptVault} from '../web-admin-v3/crypto.js';

export const KIT_TOOLS_PIN = '639ec720da11ef311471535d6383ff556839a718';
const APP_RE = /^[a-z0-9][a-z0-9-]{0,63}$/;
const HEX64 = /^[0-9a-f]{64}$/;
const FORBIDDEN = /token|secret|root|operator|sender|grant|password|\/store|v2/i;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const b64url = bytes => bytes.toString('base64').replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');

const env = name => process.env[name] || '';

function validateOrigin(raw) {
  let url;
  try { url = new URL(raw); } catch { throw Error('invalid origin URL'); }
  const loopback = ['127.0.0.1', '::1', '[::1]'].includes(url.hostname);
  if (url.protocol !== 'https:'
      && !(url.protocol === 'http:' && loopback && process.env.SKRYNIA_ALLOW_LOOPBACK_HTTP === '1')) {
    throw Error('a trusted https origin is required');
  }
  if (url.username || url.password || url.search || url.hash) throw Error('origin must not contain credentials/query/fragment');
  if (raw.startsWith('//') || url.host.startsWith('//')) throw Error('protocol-relative origin rejected');
  if (url.pathname !== '/' || raw.replace(/\/$/, '') !== url.origin) throw Error('origin must be a bare https origin');
  return url;
}

function validateEndpoint(raw, origin) {
  if (!raw.startsWith('/') || raw.startsWith('//') || raw.includes('?') || raw.includes('#')) {
    throw Error('native endpoint must be an exact same-origin absolute path');
  }
  if (raw.includes('//')) throw Error('native endpoint path traversal rejected');
  const segments = raw.split('/').filter(Boolean);
  if (segments.length === 0 || segments.some(s => s === '..' || s === '.')) {
    throw Error('native endpoint path traversal rejected');
  }
  const url = new URL(raw, origin);
  if (url.origin !== origin.origin) throw Error('native endpoint escapes the origin');
  return url.href;
}

async function readScopedCap(path, label) {
  const st = await lstat(path);
  if (st.isSymbolicLink()) throw Error(label + ' must not be a symlink');
  if (!st.isFile() || st.nlink !== 1) throw Error(label + ' must be a regular single-link file');
  if ((st.mode & 0o077) !== 0) throw Error(label + ' permissions must be 0600 or stricter');
  if (typeof process.getuid === 'function' && st.uid !== process.getuid()) throw Error(label + ' must be owned by current uid');
  if (st.size < 64 || st.size > 128) throw Error(label + ' size out of bounds');
  const fd = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const buf = Buffer.alloc(128);
    const {bytesRead} = await fd.read(buf, 0, 128, 0);
    const value = buf.subarray(0, bytesRead).toString('utf8').trim();
    if (!HEX64.test(value)) throw Error(label + ' must be exactly 32 lowercase hex bytes');
    return value;
  } finally { await fd.close(); }
}

function assertDisjoint(root, origin) {
  const roots = [env('SKRYNIA_NGINX_ROOT'), env('SKRYNIA_PUBLICATION_MAILBOX')].filter(Boolean).map(p => resolve(p));
  for (const other of roots) {
    if (root === other || root.startsWith(other + sep) || other.startsWith(root + sep)) {
      throw Error('private artifact root must be disjoint from nginx/mailbox roots');
    }
  }
  if (root === resolve('/') || root === resolve('/tmp')) throw Error('unsafe artifact root');
  if (!origin.hostname.endsWith('.test') && process.env.SKRYNIA_ALLOW_NONTEST_ORIGIN !== '1') {
    // Disposable qualification uses *.test; production origins require explicit operator review.
    throw Error('non-disposable origin requires explicit operator review flag');
  }
}

function requireInput() {
  const cfg = {
    kitPin: env('SKRYNIA_TOOLS_PIN') || KIT_TOOLS_PIN,
    artifactRoot: env('SKRYNIA_ARTIFACT_ROOT'),
    origin: env('SKRYNIA_ORIGIN_URL'),
    app: env('SKRYNIA_ADMIN_APP') || 'admin-v3',
    endpoint: env('SKRYNIA_NATIVE_ENDPOINT') || '/_skrynia/',
    readCapFile: env('SKRYNIA_NATIVE_READ_CAP_FILE'),
    adminCapFile: env('SKRYNIA_NATIVE_ADMIN_CAP_FILE'),
    adminCapsFile: env('SKRYNIA_NATIVE_ADMIN_CAPS_FILE'),
    publishHelper: env('SKRYNIA_PUBLISH_HELPER'),
    verifyHelper: env('SKRYNIA_VERIFY_HELPER'),
  };
  if (cfg.kitPin !== KIT_TOOLS_PIN) throw Error('tools source pin mismatch');
  for (const k of ['artifactRoot', 'origin', 'readCapFile', 'adminCapFile', 'publishHelper', 'verifyHelper']) {
    if (!cfg[k]) throw Error('missing required input ' + k);
  }
  if (!APP_RE.test(cfg.app)) throw Error('invalid admin application name');
  for (const k of ['readCapFile', 'adminCapFile', 'adminCapsFile', 'publishHelper', 'verifyHelper']) {
    if (cfg[k]) {
      const st = statSync(cfg[k]);
      if (!st.isFile() || st.nlink !== 1) throw Error('not a regular single-link file: ' + k);
    }
  }
  cfg.originUrl = validateOrigin(cfg.origin);
  cfg.endpointUrl = validateEndpoint(cfg.endpoint, cfg.originUrl);
  cfg.origin = cfg.originUrl.origin;
  assertDisjoint(resolve(cfg.artifactRoot), cfg.originUrl);
  return cfg;
}

async function scopedSession(cfg) {
  const readCap = await readScopedCap(cfg.readCapFile, 'native read capability');
  const adminCap = await readScopedCap(cfg.adminCapFile, 'native admin capability');
  let adminCaps = {};
  if (cfg.adminCapsFile) {
    const st = await lstat(cfg.adminCapsFile);
    if (!st.isFile() || st.nlink !== 1 || st.isSymbolicLink() || (st.mode & 0o077) !== 0) {
      throw Error('invalid admin capability map file');
    }
    adminCaps = JSON.parse(await readFile(cfg.adminCapsFile, 'utf8'));
    if (!adminCaps || typeof adminCaps !== 'object' || Array.isArray(adminCaps)) throw Error('adminCaps must be an object');
    for (const [app, cap] of Object.entries(adminCaps)) {
      if (!APP_RE.test(app) || !HEX64.test(cap)) throw Error('invalid scoped admin capability');
    }
  }
  const session = {schema: 1, native: {endpoint: cfg.endpointUrl, readCap, adminCap, adminCaps}};
  if (FORBIDDEN.test(JSON.stringify(session.native))) throw Error('vault would contain non-scoped authority');
  return session;
}

async function runHelper(bin, payload) {
  const {spawnSync} = await import('node:child_process');
  const cp = spawnSync(bin, [], {input: JSON.stringify(payload), encoding: 'utf8',
    timeout: 300000, stdio: ['pipe', 'pipe', 'pipe']});
  if (cp.error || cp.status !== 0) {
    // Never forward helper stderr: it may contain operator token material.
    throw Error(`publication helper ${bin} failed (exit ${cp.status ?? 'signal'}; stderr withheld)`);
  }
  let parsed;
  try { parsed = JSON.parse(cp.stdout); } catch { throw Error('publication helper returned non-JSON result'); }
  if (!parsed || parsed.ok !== true) throw Error('publication helper reported failure');
  return parsed;
}

export async function install(cfg = requireInput()) {
  if (!cfg || cfg.kitPin !== KIT_TOOLS_PIN) throw Error('tools source pin mismatch');
  const origin = cfg.originUrl ? cfg.originUrl.origin : validateOrigin(cfg.origin).origin;
  const endpoint = cfg.endpointUrl || validateEndpoint(cfg.endpoint || '/_skrynia/', new URL(origin));
  const session = await scopedSession({...cfg, originUrl: new URL(origin), endpointUrl: endpoint});
  const root = resolve(cfg.artifactRoot);
  assertDisjoint(root, new URL(origin));
  const appRoot = join(root, 'apps', cfg.app);
  await mkdir(appRoot, {recursive: true});
  await writeFile(join(appRoot, '.installed'),
    JSON.stringify({kitPin: cfg.kitPin, created: new Date().toISOString()}) + '\n',
    {flag: 'wx', mode: 0o600});

  const password = b64url(randomBytes(32));
  const vaultBytes = await encryptVault(session, password, 'browser-admin-authority');

  const assetsDir = resolve(fileURLToPath(new URL('../web-admin-v3/', import.meta.url)));
  const files = [];
  for (const name of await readdir(assetsDir)) {
    if (!/\.(js|html|css)$/.test(name)) continue;
    const bytes = await readFile(join(assetsDir, name));
    files.push({rel: name, bytes, sha256: sha256(bytes)});
  }
  const unlockBytes = Buffer.from((await readFile(new URL('./unlock.html', import.meta.url))).toString()
    .replaceAll('__APP__', cfg.app));
  files.push({rel: 'unlock.html', bytes: unlockBytes, sha256: sha256(unlockBytes)});
  files.push({rel: 'bootstrap.vault.json', bytes: vaultBytes, sha256: sha256(vaultBytes)});
  const roster = files.map(f => ({rel: f.rel, sha256: f.sha256})).sort((a, b) => a.rel.localeCompare(b.rel));
  const release = sha256(Buffer.from(JSON.stringify(roster)));
  const manifest = {schema: 'skrynia-admin-bundle-v1', app: cfg.app, release, files: roster};
  const manifestBytes = Buffer.from(JSON.stringify(manifest, null, 2) + '\n');
  files.push({rel: 'manifest.json', bytes: manifestBytes, sha256: sha256(manifestBytes)});

  // Layout under artifact root matches publication rel paths:
  //   apps/<app>/<release>/<file>  (immutable)
  //   apps/<app>/<file>            (stable app entry)
  // Both are public-safe static assets (0644). Private caps never copied here.
  const releaseDir = join(appRoot, release);
  await mkdir(releaseDir, {recursive: false});
  for (const f of files) {
    await writeFile(join(releaseDir, f.rel), f.bytes, {flag: 'wx', mode: 0o644});
    await writeFile(join(appRoot, f.rel), f.bytes, {flag: 'wx', mode: 0o644});
  }

  const releaseFiles = files.map(f => ({
    rel: `apps/${cfg.app}/${release}/${f.rel}`, abs: join(releaseDir, f.rel), sha256: f.sha256,
  }));
  const stableFiles = files.map(f => ({
    rel: `apps/${cfg.app}/${f.rel}`, abs: join(appRoot, f.rel), sha256: f.sha256,
  }));
  const payload = {
    app: cfg.app, release, kitPin: cfg.kitPin, origin, endpoint,
    files: releaseFiles,
    stable: stableFiles,
  };
  const published = await runHelper(cfg.publishHelper, {...payload, action: 'publish'});
  if (published.release !== release) throw Error('publish helper release mismatch');
  const verified = await runHelper(cfg.verifyHelper, {...payload, action: 'verify'});
  if (verified.release !== release || !Array.isArray(verified.files)) throw Error('verify helper result malformed');
  const expected = new Map([...payload.files, ...payload.stable].map(f => [f.rel, f.sha256]));
  if (verified.files.length !== expected.size) throw Error('served-file roster mismatch');
  for (const entry of verified.files) {
    if (expected.get(entry.rel) !== entry.sha256 || entry.status !== 200) {
      throw Error('served bytes mismatch for ' + entry.rel);
    }
  }
  // Stable entry, not a version-specific URL only.
  return {link: `${origin}/apps/${cfg.app}/unlock.html`, password, release};
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  install().then(({link, password, release}) => {
    process.stdout.write('ADMIN_LINK=' + link + '\n');
    process.stdout.write('ADMIN_RELEASE=' + release + '\n');
    process.stdout.write('ADMIN_ACCESS_PASSWORD=' + password + '\n');
    process.stdout.write('Store the password now; it is not written to disk in plaintext.\n');
  }).catch(e => {
    process.stderr.write('installer failed closed: ' + (e.message || String(e)) + '\n');
    process.exitCode = 1;
  });
}
