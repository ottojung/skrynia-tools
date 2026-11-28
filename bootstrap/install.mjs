#!/usr/bin/env node
// One-time native-v3 trusted-host admin bootstrap. No subcommands, no CLI.
// Publishes immutable admin assets through the operator's already-deployed
// publication root/provider helper and prints link + password only after every
// served file has been independently SHA-256 verified.
import {createHash, randomBytes} from 'node:crypto';
import {constants, statSync} from 'node:fs';
import {lstat, mkdir, open, readFile, readdir, realpath, writeFile} from 'node:fs/promises';
import {join, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {encryptVault} from '../web-admin-v3/crypto.js';

// Exact bootstrap bundle identity = SHA-256 of bootstrap/SHA256SUMS, verified
// out-of-band by deploy.sh (or an equivalent trusted operator flow) and passed
// here as SKRYNIA_BOOTSTRAP_BUNDLE_ID. SHA256SUMS does not list itself, so
// this anchor is not self-referential. It identifies the exact bundle bytes,
// not the running kernel/tools stack (qualified by exact-stack CI separately).
export const BUNDLE_ID_RE = /^[0-9a-f]{64}$/;
const APP_RE = /^[a-z0-9][a-z0-9-]{0,63}$/;
const HEX64 = /^[0-9a-f]{64}$/;
const FORBIDDEN = /token|secret|root|operator|sender|grant|password|\/store|v2/i;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const b64url = bytes => bytes.toString('base64').replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');

const env = name => process.env[name] || '';

async function rejectSymlinkParents(p) {
  let cur = resolve(p);
  const root = resolve('/');
  while (true) {
    try {
      const st = await lstat(cur);
      if (st.isSymbolicLink()) throw Error('symlink in path rejected: ' + cur);
    } catch (e) {
      if (e && e.code !== 'ENOENT') throw e;
      // Missing tail is allowed; check existing parents.
    }
    if (cur === root) break;
    const parent = resolve(cur, '..');
    if (parent === cur) break;
    cur = parent;
  }
}

async function assertFdStat(fd, st, label) {
  const fst = await fd.stat();
  if (!fst.isFile() || fst.isSymbolicLink()) throw Error(label + ' fd is not a regular file');
  if (fst.nlink !== st.nlink || fst.uid !== st.uid || fst.size !== st.size || (fst.mode & 0o777) !== (st.mode & 0o777)) {
    throw Error(label + ' changed between lstat and fstat');
  }
}

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
  await rejectSymlinkParents(path);
  const st = await lstat(path);
  if (st.isSymbolicLink()) throw Error(label + ' must not be a symlink');
  if (!st.isFile() || st.nlink !== 1) throw Error(label + ' must be a regular single-link file');
  if ((st.mode & 0o077) !== 0) throw Error(label + ' permissions must be 0600 or stricter');
  if (typeof process.getuid === 'function' && st.uid !== process.getuid()) throw Error(label + ' must be owned by current uid');
  if (st.size < 64 || st.size > 128) throw Error(label + ' size out of bounds');
  const fd = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    await assertFdStat(fd, st, label);
    const buf = Buffer.alloc(128);
    const {bytesRead} = await fd.read(buf, 0, 128, 0);
    const value = buf.subarray(0, bytesRead).toString('utf8').trim();
    if (!HEX64.test(value)) throw Error(label + ' must be exactly 32 lowercase hex bytes');
    return value;
  } finally { await fd.close(); }
}

function assertDisposableOrigin(origin) {
  if (!origin.hostname.endsWith('.test') && process.env.SKRYNIA_ALLOW_NONTEST_ORIGIN !== '1') {
    // Disposable qualification uses *.test; production origins require explicit operator review.
    throw Error('non-disposable origin requires explicit operator review flag');
  }
}

// The private artifact root and the two documented external roots must be
// existing, non-aliased directories. Canonical paths (not lexical env strings)
// are compared so bind/symlink aliases and parent/child nesting are rejected.
async function assertPrivateArtifactRoot(rawRoot, cfg) {
  const root = resolve(rawRoot);
  if (root === resolve('/') || root === resolve('/tmp')) throw Error('unsafe artifact root');
  await rejectSymlinkParents(root);
  let st;
  try { st = await lstat(root); } catch (e) {
    if (e && e.code === 'ENOENT') throw Error('artifact root must be an existing real directory');
    throw e;
  }
  if (st.isSymbolicLink() || !st.isDirectory()) throw Error('artifact root must be an existing real directory');
  if ((st.mode & 0o077) !== 0) throw Error('artifact root permissions must be 0700 or stricter');
  if (typeof process.getuid === 'function' && st.uid !== process.getuid()) throw Error('artifact root must be owned by current uid');
  const canonical = await realpath(root);
  for (const [label, value] of [['nginx document root', cfg.nginxRoot], ['publication mailbox', cfg.mailbox]]) {
    if (!value) throw Error('missing ' + label + ' prerequisite');
    const other = resolve(value);
    await rejectSymlinkParents(other);
    let otherSt;
    try { otherSt = await lstat(other); } catch { throw Error(label + ' must be an existing real directory'); }
    if (!otherSt.isDirectory() || otherSt.isSymbolicLink()) throw Error(label + ' must be an existing real directory');
    const otherCanonical = await realpath(other);
    if (canonical === otherCanonical || canonical.startsWith(otherCanonical + sep) || otherCanonical.startsWith(canonical + sep)) {
      throw Error('private artifact root must be disjoint from ' + label);
    }
    // Exact same-directory bind mounts share dev+ino and are rejected. Distinct
    // bind-mount aliases cannot be detected portably from userspace; operators
    // must not supply aliased mount paths (documented trusted-host assumption).
    if (st.dev === otherSt.dev && st.ino === otherSt.ino) {
      throw Error('private artifact root must be disjoint from ' + label);
    }
  }
}

function requireInput() {
  const cfg = {
    bundleId: env('SKRYNIA_BOOTSTRAP_BUNDLE_ID'),
    artifactRoot: env('SKRYNIA_ARTIFACT_ROOT'),
    nginxRoot: env('SKRYNIA_NGINX_ROOT'),
    mailbox: env('SKRYNIA_PUBLICATION_MAILBOX'),
    origin: env('SKRYNIA_ORIGIN_URL'),
    app: env('SKRYNIA_ADMIN_APP') || 'admin-v3',
    endpoint: env('SKRYNIA_NATIVE_ENDPOINT') || '/_skrynia/',
    readCapFile: env('SKRYNIA_NATIVE_READ_CAP_FILE'),
    adminCapFile: env('SKRYNIA_NATIVE_ADMIN_CAP_FILE'),
    adminCapsFile: env('SKRYNIA_NATIVE_ADMIN_CAPS_FILE'),
    publishHelper: env('SKRYNIA_PUBLISH_HELPER'),
    verifyHelper: env('SKRYNIA_VERIFY_HELPER'),
  };
  if (!BUNDLE_ID_RE.test(cfg.bundleId || '')) throw Error('missing or invalid out-of-band bootstrap bundle identity (SKRYNIA_BOOTSTRAP_BUNDLE_ID)');
  for (const k of ['artifactRoot', 'nginxRoot', 'mailbox', 'origin', 'readCapFile', 'adminCapFile', 'publishHelper', 'verifyHelper']) {
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
  return cfg;
}

async function scopedSession(cfg) {
  const readCap = await readScopedCap(cfg.readCapFile, 'native read capability');
  const adminCap = await readScopedCap(cfg.adminCapFile, 'native admin capability');
  let adminCaps = {};
  if (cfg.adminCapsFile) {
    await rejectSymlinkParents(cfg.adminCapsFile);
    const st = await lstat(cfg.adminCapsFile);
    if (!st.isFile() || st.nlink !== 1 || st.isSymbolicLink() || (st.mode & 0o077) !== 0) {
      throw Error('invalid admin capability map file');
    }
    if (typeof process.getuid === 'function' && st.uid !== process.getuid()) throw Error('admin capability map must be owned by current uid');
    if (st.size < 2 || st.size > 65536) throw Error('admin capability map size out of bounds');
    const fd = await open(cfg.adminCapsFile, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
    let raw;
    try {
      await assertFdStat(fd, st, 'admin capability map');
      raw = (await fd.readFile({encoding: 'utf8'}));
    } finally { await fd.close(); }
    adminCaps = JSON.parse(raw);
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
  if (!cfg || !BUNDLE_ID_RE.test(cfg.bundleId || '')) throw Error('missing or invalid out-of-band bootstrap bundle identity (SKRYNIA_BOOTSTRAP_BUNDLE_ID)');
  const origin = cfg.originUrl ? cfg.originUrl.origin : validateOrigin(cfg.origin).origin;
  const endpoint = cfg.endpointUrl || validateEndpoint(cfg.endpoint || '/_skrynia/', new URL(origin));
  assertDisposableOrigin(new URL(origin));
  const session = await scopedSession({...cfg, originUrl: new URL(origin), endpointUrl: endpoint});
  const root = resolve(cfg.artifactRoot);
  await assertPrivateArtifactRoot(root, cfg);
  const appRoot = join(root, 'apps', cfg.app);
  await mkdir(appRoot, {recursive: true});
  await rejectSymlinkParents(appRoot);
  await writeFile(join(appRoot, '.installed'),
    JSON.stringify({bundleId: cfg.bundleId, created: new Date().toISOString()}) + '\n',
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
    app: cfg.app, release, bundleId: cfg.bundleId, origin, endpoint,
    files: releaseFiles,
    stable: stableFiles,
  };
  const expected = new Map([...payload.files, ...payload.stable].map(f => [f.rel, f.sha256]));
  let publishedOk = false;
  try {
    const published = await runHelper(cfg.publishHelper, {...payload, action: 'publish'});
    if (published.release !== release || published.action !== 'publish') throw Error('publish helper release mismatch');
    // A publish helper can report ok only after every requested op 1 returned
    // the definitive all-or-nothing zero code. Unknown outcomes must be thrown
    // by the helper and are never treated as published.
    publishedOk = true;
    const verified = await runHelper(cfg.verifyHelper, {...payload, action: 'verify'});
    if (verified.release !== release || !Array.isArray(verified.files)) throw Error('verify helper result malformed');
    if (verified.origin !== origin) throw Error('verify helper origin mismatch');
    const seen = new Set();
    for (const entry of verified.files) {
      if (!entry || typeof entry.rel !== 'string') throw Error('verify helper result malformed');
      if (seen.has(entry.rel)) throw Error('duplicate served-file verification result for ' + entry.rel);
      seen.add(entry.rel);
    }
    if (seen.size !== expected.size) throw Error('served-file roster mismatch');
    for (const rel of expected.keys()) {
      if (!seen.has(rel)) throw Error('served-file roster missing ' + rel);
    }
    for (const entry of verified.files) {
      if (expected.get(entry.rel) !== entry.sha256 || entry.status !== 200) {
        throw Error('served bytes mismatch for ' + entry.rel);
      }
    }
  } catch (error) {
    // Never blindly unpublish after an unknown publish outcome or ambiguous
    // verification: revoking a path not proven owned/created by this run is
    // unsafe. Only when every publish op returned definitive success do we
    // undo through the SAME native publish root (op 2). Otherwise stop and
    // require operator reconciliation; no password is printed.
    if (!publishedOk) {
      throw Error(`${error.message || String(error)} (publish outcome unknown or failed before definitive success; no unpublish attempted; operator reconciliation required)`);
    }
    let rollback = 'unpublish completed';
    try {
      const undone = await runHelper(cfg.publishHelper, {...payload, action: 'unpublish'});
      if (undone.action !== 'unpublish') throw Error('unpublish helper result malformed');
    } catch {
      rollback = 'UNPUBLISH FAILED: public state uncertain; operator must reconcile';
    }
    throw Error(`${error.message || String(error)} (${rollback})`);
  }
  // Stable entry, not a version-specific URL only.
  const link = `${origin}/apps/${cfg.app}/unlock.html`;
  const pwFile = join(appRoot, `password.${release}.txt`);
  // Private recovery record: only after successful publication+verification.
  await writeFile(pwFile, password + '\n', {flag: 'wx', mode: 0o600});
  return {link, password, release, passwordFile: pwFile};
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  install().then(({link, password, release, passwordFile}) => {
    process.stdout.write('ADMIN_LINK=' + link + '\n');
    process.stdout.write('ADMIN_RELEASE=' + release + '\n');
    process.stdout.write('ADMIN_ACCESS_PASSWORD=' + password + '\n');
    process.stdout.write('ADMIN_PASSWORD_FILE=' + passwordFile + '\n');
    process.stdout.write('Password written to private 0600 file after verified publication; keep it private.\n');
  }).catch(e => {
    process.stderr.write('installer failed closed: ' + (e.message || String(e)) + '\n');
    process.exitCode = 1;
  });
}
