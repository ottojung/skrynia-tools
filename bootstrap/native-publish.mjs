#!/usr/bin/env node
// Narrow native publication helper for the one-time admin bootstrap.
// It NEVER writes nginx files. `publish` and `unpublish` invoke the operator's
// compiled publication root WASM (kernel POST, op 1 / op 2) which calls the
// reference provider; `verify` independently GETs each served file and hashes
// its bytes. Side effects stay at this boundary (injected operator helper);
// no capability material is ever echoed on stdout or stderr.
import {createHash} from 'node:crypto';
import {constants} from 'node:fs';
import {lstat, open, readFile} from 'node:fs/promises';
import http from 'node:http';
import https from 'node:https';
import {join, resolve, sep} from 'node:path';

const HEX64 = /^[0-9a-f]{64}$/;
const OP_PUBLISH = 1;
const OP_UNPUBLISH = 2;
const TIMEOUT = 10000;
const sha256 = b => createHash('sha256').update(b).digest('hex');

async function readStdin() {
  let raw = '';
  for await (const chunk of process.stdin) raw += chunk.toString('utf8');
  return raw;
}

function publicPath(rel, app) {
  if (!rel.startsWith(`apps/${app}/`)) throw Error('path outside admin app namespace');
  if (rel.includes('\\') || rel.includes('\0') || rel.includes('\n') || rel.includes('\r')
      || rel.endsWith('/') || rel.split('/').some(p => !p || p.startsWith('.'))
      || Buffer.byteLength(rel, 'utf8') > 4096) throw Error('unsafe publication path');
  const nginxRoot = process.env.SKYRNIA_NGINX_ROOT;
  if (nginxRoot) {
    const root = resolve(nginxRoot), out = resolve(join(root, rel));
    if (out !== root && !out.startsWith(root + sep)) throw Error('path escapes nginx root');
  }
  return rel;
}

function urlFromEnv(name) {
  const raw = process.env[name] || '';
  let u;
  try { u = new URL(raw); } catch { throw Error('invalid ' + name); }
  if (u.username || u.password || u.search || u.hash) throw Error(name + ' must not contain credentials/query/fragment');
  if (!['http:', 'https:'].includes(u.protocol)) throw Error(name + ' must be http(s)');
  return u;
}

function publicationUrl() {
  const u = urlFromEnv('SKRYNIA_PUBLICATION_URL');
  const loopback = ['127.0.0.1', '::1', '[::1]'].includes(u.hostname);
  if (u.protocol === 'http:' && !loopback) throw Error('plain HTTP publication only on literal loopback');
  if (u.protocol === 'https:' && process.env.SKYRNIA_PUBLICATION_ALLOW_TLS !== '1') {
    throw Error('https publication endpoint requires explicit review flag');
  }
  return u;
}

async function token() {
  const file = process.env.SKYRNIA_PUBLICATION_TOKEN_FILE;
  if (!file) throw Error('SKRYNIA_PUBLICATION_TOKEN_FILE is required');
  const st = await lstat(file);
  if (st.isSymbolicLink() || !st.isFile() || st.nlink !== 1) throw Error('invalid token file');
  if ((st.mode & 0o077) !== 0) throw Error('token file permissions too broad');
  if (typeof process.getuid === 'function' && st.uid !== process.getuid()) throw Error('token file wrong owner');
  if (st.size < 64 || st.size > 128) throw Error('token file size out of bounds');
  const fd = await open(file, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const buf = Buffer.alloc(128);
    const {bytesRead} = await fd.read(buf, 0, 128, 0);
    const value = buf.subarray(0, bytesRead).toString('utf8').trim();
    if (!HEX64.test(value)) throw Error('invalid operator token');
    return Buffer.from(value, 'hex');
  } finally { await fd.close(); }
}

function callRoot(base, tok, op, rel) {
  const raw = Buffer.from(rel, 'utf8');
  const frame = Buffer.allocUnsafe(1 + tok.length + 1 + 4 + raw.length);
  let at = 0;
  frame.writeUInt8(1, at); at += 1;
  tok.copy(frame, at); at += tok.length;
  frame.writeUInt8(op, at); at += 1;
  frame.writeUInt32LE(raw.length, at); at += 4;
  raw.copy(frame, at);
  const mod = base.protocol === 'https:' ? https : http;
  return new Promise(done => {
    let settled = false;
    const finish = v => { if (!settled) { settled = true; clearTimeout(timer); done(v); } };
    const req = mod.request(base, {method: 'POST', headers: {
      'Content-Type': 'application/octet-stream', 'Content-Length': frame.length}}, res => {
      const chunks = []; let total = 0;
      res.on('data', c => { total += c.length; if (total <= 4) chunks.push(c); else finish({ok: false}); });
      res.on('aborted', () => finish({ok: false}));
      res.on('close', () => {
        if (settled) return;
        const body = Buffer.concat(chunks);
        finish(res.statusCode === 200 && total === 4 && body.length === 4 && body.readInt32LE(0) === 0
          ? {ok: true} : {ok: false});
      });
    });
    const timer = setTimeout(() => { req.destroy(); finish({ok: false}); }, TIMEOUT);
    req.on('error', () => finish({ok: false}));
    req.end(frame);
  });
}

function getServed(base, rel, app) {
  publicPath(rel, app); // re-validate before constructing URL
  const url = new URL('/' + rel.split('/').map(encodeURIComponent).join('/'), base);
  if (url.origin !== base.origin) throw Error('verification URL escaped origin');
  const mod = url.protocol === 'https:' ? https : http;
  return new Promise((done, reject) => {
    let settled = false;
    const finish = (e, v) => { if (!settled) { settled = true; clearTimeout(timer); e ? reject(e) : done(v); } };
    const req = mod.get(url, res => {
      const h = createHash('sha256'); let total = 0;
      res.on('data', c => { total += c.length; if (total <= 32 * 1024 * 1024) h.update(c); });
      res.on('aborted', () => finish(Error('aborted')));
      res.on('close', () => {
        if (settled) return;
        if (!res.complete) return finish(Error('incomplete'));
        finish(null, {status: res.statusCode, sha256: h.digest('hex'), bytes: total});
      });
    });
    const timer = setTimeout(() => { req.destroy(); finish(Error('deadline')); }, TIMEOUT);
    req.on('error', finish);
  });
}

async function main() {
  const input = JSON.parse(await readStdin());
  if (!input || !['publish', 'unpublish', 'verify'].includes(input.action)) throw Error('invalid action');
  if (!HEX64.test(input.release || '')) throw Error('invalid release');
  const app = input.app;
  if (!/^[a-z0-9][a-z0-9-]{0,63}$/.test(app)) throw Error('invalid app');
  if (!Array.isArray(input.files)) throw Error('invalid files roster');

  if (input.action === 'verify') {
    const base = urlFromEnv('SKRYNIA_PUBLIC_BASE_URL');
    if (base.pathname !== '/') throw Error('public base must be an origin');
    const out = [];
    for (const f of [...input.files, ...(input.stable || [])]) {
      const rel = publicPath(f.rel, app);
      const got = await getServed(base, rel, app);
      out.push({rel, status: got.status, sha256: got.sha256, bytes: got.bytes});
    }
    process.stdout.write(JSON.stringify({ok: true, release: input.release, files: out}) + '\n');
    return;
  }

  const base = publicationUrl();
  const tok = await token();
  const op = input.action === 'unpublish' ? OP_UNPUBLISH : OP_PUBLISH;
  for (const f of [...input.files, ...(input.stable || [])]) {
    const rel = publicPath(f.rel, app);
    if (op === OP_PUBLISH) {
      const expected = f.sha256;
      if (!HEX64.test(expected || '')) throw Error('invalid expected hash');
      const abs = resolve(f.abs);
      const artifactRoot = process.env.SKRYNIA_ARTIFACT_ROOT || '';
      if (artifactRoot && !(abs === resolve(artifactRoot) || abs.startsWith(resolve(artifactRoot) + sep))) {
        throw Error('source file outside private artifact root');
      }
      const st = await lstat(abs);
      if (!st.isFile() || st.isSymbolicLink()) throw Error('source must be a regular file');
      const bytes = await readFile(abs);
      if (sha256(bytes) !== expected) throw Error('source bytes changed before publication');
    }
    // Both ops require a complete zero-code reply; unknown/malformed replies fail closed.
    const result = await callRoot(base, tok, op, rel);
    if (!result.ok) throw Error((op === OP_PUBLISH ? 'publish' : 'unpublish') + ' rejected ' + rel);
  }
  process.stdout.write(JSON.stringify({ok: true, release: input.release, action: input.action}) + '\n');
}

main().catch(() => {
  // No stderr detail: helper output must never echo token or capability data.
  process.stderr.write('native publication helper failed closed\n');
  process.exitCode = 1;
});
