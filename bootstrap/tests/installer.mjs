import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, mkdir, readFile, rm, writeFile, chmod} from 'node:fs/promises';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {createHash} from 'node:crypto';
import {decryptVault} from '../../web-admin-v3/crypto.js';
import {install, KIT_TOOLS_PIN} from '../install.mjs';

const hex = n => n.repeat(64);
const sha = b => createHash('sha256').update(b).digest('hex');

async function fixture(t) {
  const dir = await mkdtemp(join(tmpdir(), 'bootstrap-install-'));
  t.after(() => rm(dir, {recursive: true, force: true}));
  const root = join(dir, 'private-root');
  await mkdir(root);
  const readCap = join(dir, 'read.cap'), adminCap = join(dir, 'admin.cap');
  await writeFile(readCap, hex('a'), {mode: 0o600});
  await writeFile(adminCap, hex('b'), {mode: 0o600});
  const helperMod = join(dir, 'helper.mjs');
  await writeFile(helperMod, `
let raw = '';
for await (const chunk of process.stdin) raw += chunk.toString('utf8');
const j = JSON.parse(raw);
if (j.action === 'verify') {
  const files = [...j.files, ...(j.stable || [])].map(f => ({rel: f.rel, status: 200, sha256: f.sha256}));
  process.stdout.write(JSON.stringify({ok: true, release: j.release, files}) + '\\n');
} else process.stdout.write(JSON.stringify({ok: true, release: j.release}) + '\\n');
`);
  const wrap = join(dir, 'helper.sh');
  await writeFile(wrap, `#!/bin/sh\nexec node '${helperMod}'\n`);
  await chmod(wrap, 0o700);
  const cfg = {
    kitPin: KIT_TOOLS_PIN,
    artifactRoot: root,
    origin: 'https://admin.example.test',
    app: 'admin-v3',
    endpoint: '/_skrynia/',
    readCapFile: readCap,
    adminCapFile: adminCap,
    adminCapsFile: '',
    publishHelper: wrap,
    verifyHelper: wrap,
  };
  return {dir, cfg, root};
}

test('install refuses a divergent tools kit pin and unsafe origins/endpoints', async t => {
  const {cfg} = await fixture(t);
  await assert.rejects(install({...cfg, kitPin: 'f'.repeat(40)}), /tools source pin mismatch/);
  await assert.rejects(install({...cfg, origin: 'http://admin.example.test'}), /https origin/);
  await assert.rejects(install({...cfg, origin: 'https://u:p@admin.example.test/'}), /credentials/);
  await assert.rejects(install({...cfg, endpoint: '//evil.test/x'}), /same-origin/);
  await assert.rejects(install({...cfg, endpoint: '/_skrynia/../secret'}), /traversal/);
});

test('install refuses invalid native authority files', async t => {
  const {dir, cfg} = await fixture(t);
  await writeFile(join(dir, 'bad.cap'), 'not-hex', {mode: 0o600});
  await assert.rejects(install({...cfg, readCapFile: join(dir, 'bad.cap')}), /capability/);
  await writeFile(join(dir, 'bad.json'), JSON.stringify({app: 'root-token', cap: hex('c')}), {mode: 0o600});
  await assert.rejects(install({...cfg, adminCapsFile: join(dir, 'bad.json')}), /invalid scoped admin/);
  const worldReadable = join(dir, 'loose.cap');
  await writeFile(worldReadable, hex('d'), {mode: 0o644});
  await assert.rejects(install({...cfg, readCapFile: worldReadable}), /permissions/);
});

test('install refuses to overwrite an existing private output', async t => {
  const {cfg} = await fixture(t);
  const first = await install(cfg);
  await assert.rejects(install(cfg), /EEXIST|exists/);
  assert.ok(first.release);
});

test('vault is scoped and published assets stay public-safe', async t => {
  const {cfg, root} = await fixture(t);
  const result = await install(cfg);
  assert.match(result.password, /^[A-Za-z0-9_-]{43}$/);
  assert.equal(result.link, `${cfg.origin}/apps/admin-v3/unlock.html`);
  const releaseDir = join(root, 'apps', 'admin-v3', result.release);
  const vaultBytes = await readFile(join(releaseDir, 'bootstrap.vault.json'));
  const vault = await decryptVault(vaultBytes, result.password, 'browser-admin-authority');
  assert.deepEqual(Object.keys(vault), ['schema', 'native']);
  assert.deepEqual(Object.keys(vault.native), ['endpoint', 'readCap', 'adminCap', 'adminCaps']);
  assert.equal(vault.native.readCap, hex('a'));
  assert.equal(vault.native.endpoint, 'https://admin.example.test/_skrynia/');
  const raw = JSON.stringify(vault).toLowerCase();
  for (const forbidden of ['token', 'operator', 'sender', 'root', 'grant', '/store', 'v2']) {
    assert.equal(raw.includes(forbidden), false, 'vault leaked ' + forbidden);
  }
  for (const name of ['unlock.html', 'index.html', 'main.js', 'native_bootstrap.js', 'manifest.json']) {
    const bytes = await readFile(join(releaseDir, name));
    assert.equal(bytes.includes(Buffer.from(hex('a'))), false, name + ' leaks read cap');
    assert.equal(bytes.includes(Buffer.from(result.password)), false, name + ' leaks password');
  }
  const manifest = JSON.parse(await readFile(join(releaseDir, 'manifest.json'), 'utf8'));
  for (const f of manifest.files) {
    assert.equal(sha(await readFile(join(releaseDir, f.rel))), f.sha256, f.rel + ' hash');
  }
});
