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
  const actionsFile = join(dir, 'helper-actions.log');
  await writeFile(helperMod, `
import {appendFile} from 'node:fs/promises';
let raw = '';
for await (const chunk of process.stdin) raw += chunk.toString('utf8');
const j = JSON.parse(raw);
await appendFile(${JSON.stringify(actionsFile)}, j.action + '\\n');
if (j.action === 'verify') {
  if (process.env.FAIL_VERIFY === '1') {
    process.stdout.write(JSON.stringify({ok: true, release: j.release, origin: j.origin, files: []}) + '\\n');
  } else {
    const files = [...j.files, ...(j.stable || [])].map(f => ({rel: f.rel, status: 200, sha256: f.sha256}));
    process.stdout.write(JSON.stringify({ok: true, release: j.release, origin: j.origin, files}) + '\\n');
  }
} else process.stdout.write(JSON.stringify({ok: true, release: j.release, action: j.action}) + '\\n');
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

test('failed served-byte verification unpublishes and prints no authority', async t => {
  const {cfg, dir} = await fixture(t);
  // The mock verify helper returns an empty roster when FAIL_VERIFY=1.
  const verifyShell = join(dir, 'verify-fail.sh');
  await writeFile(verifyShell, `#!/bin/sh\nexport FAIL_VERIFY=1\nexec node '${join(dir, 'helper.mjs')}'\n`);
  await chmod(verifyShell, 0o700);
  await assert.rejects(install({...cfg, verifyHelper: verifyShell}), /served-file roster mismatch.*unpublish completed/);
  const actions = await readFile(join(dir, 'helper-actions.log'), 'utf8');
  assert.match(actions, /^publish\nverify\nunpublish\n$/, 'publish/verify/unpublish order');
});

test('verify rejects public-base origin mismatch', async t => {
  const {cfg, dir} = await fixture(t);
  const bad = join(dir, 'verify-bad-origin.mjs');
  await writeFile(bad, `
let raw=''; for await (const c of process.stdin) raw+=c;
const j=JSON.parse(raw);
if (j.action==='verify') {
  const files=[...j.files,...(j.stable||[])].map(f=>({rel:f.rel,status:200,sha256:f.sha256}));
  process.stdout.write(JSON.stringify({ok:true,release:j.release,origin:'https://evil.test',files})+'\\n');
} else process.stdout.write(JSON.stringify({ok:true,release:j.release,action:j.action})+'\\n');
`);
  const wrap = join(dir, 'verify-bad-origin.sh');
  await writeFile(wrap, `#!/bin/sh\nexec node '${bad}'\n`);
  await chmod(wrap, 0o700);
  await assert.rejects(install({...cfg, verifyHelper: wrap}), /origin mismatch.*unpublish completed/);
});

test('SHA256SUMS covers all published web-admin-v3 assets', async () => {
  const {readdir, readFile} = await import('node:fs/promises');
  const sums = await readFile(new URL('../SHA256SUMS', import.meta.url), 'utf8');
  const assets = (await readdir(new URL('../../web-admin-v3/', import.meta.url)))
    .filter(n => /\.(js|html|css)$/.test(n)).sort();
  for (const name of assets) {
    assert.match(sums, new RegExp(`\\.\\./web-admin-v3/${name.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')}$`, 'm'),
      'SHA256SUMS missing ' + name);
  }
  assert.match(sums, /\.\.\/web-admin-v3\/crypto\.js$/m, 'crypto.js must be covered');
});

test('sha256sum -c detects a byte mutation of a published JS asset', async t => {
  const {cp} = await import('node:fs/promises');
  const {spawnSync} = await import('node:child_process');
  const dir = await mkdtemp(join(tmpdir(), 'bootstrap-sums-'));
  t.after(() => rm(dir, {recursive: true, force: true}));
  await cp(new URL('../', import.meta.url), join(dir, 'bootstrap'), {recursive: true});
  await cp(new URL('../../web-admin-v3/', import.meta.url), join(dir, 'web-admin-v3'), {recursive: true});
  const cwd = join(dir, 'bootstrap');
  const check = () => spawnSync('sha256sum', ['-c', 'SHA256SUMS'], {cwd, encoding: 'utf8'});
  const clean = check();
  assert.equal(clean.status, 0, 'unmodified bundle must verify: ' + clean.stderr);
  assert.equal((clean.stdout.match(/: OK$/gm) || []).length, 17, '17 OK lines');
  const asset = join(dir, 'web-admin-v3', 'main.js');
  const mutated = Buffer.from(await readFile(asset));
  mutated[0] ^= 0x01; // flip exactly one byte of a published JS asset
  await writeFile(asset, mutated);
  const bad = check();
  assert.notEqual(bad.status, 0, 'mutated JS asset must fail sha256sum -c');
  assert.match(bad.stdout + bad.stderr, /main\.js: FAILED/, 'detects main.js mutation');
});
