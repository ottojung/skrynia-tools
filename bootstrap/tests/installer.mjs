import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, mkdir, readFile, readdir, rm, writeFile, chmod, symlink, stat} from 'node:fs/promises';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {createHash} from 'node:crypto';
import {decryptVault} from '../../web-admin-v3/crypto.js';
import {install, BUNDLE_ID_RE} from '../install.mjs';

const hex = n => n.repeat(64);
const sha = b => createHash('sha256').update(b).digest('hex');
const byRel = (a, b) => a.localeCompare(b);
const WEB_ASSETS = [
  'apps.js', 'apps_ui.js', 'binary_transport.js', 'browser_storage.js',
  'capabilities.js', 'crypto.js', 'index.html', 'main.js', 'native_bootstrap.js',
  'native_capabilities.js', 'styles.css', 'vault.js',
];

async function fixture(t) {
  const dir = await mkdtemp(join(tmpdir(), 'bootstrap-install-'));
  t.after(() => rm(dir, {recursive: true, force: true}));
  const root = join(dir, 'private-root');
  const nginxRoot = join(dir, 'nginx-root');
  const mailbox = join(dir, 'publication-mailbox');
  for (const p of [root, nginxRoot, mailbox]) {
    await mkdir(p, {mode: 0o700});
    await chmod(p, 0o700);
  }
  // Coordinate the fixture environment: the disjoint roots are explicit inputs,
  // never inherited from whatever the host happens to have configured.
  for (const [key, value] of [['SKRYNIA_NGINX_ROOT', nginxRoot], ['SKRYNIA_PUBLICATION_MAILBOX', mailbox]]) {
    const saved = process.env[key];
    process.env[key] = value;
    t.after(() => {
      if (saved === undefined) delete process.env[key];
      else process.env[key] = saved;
    });
  }
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
if (j.action === 'publish' && process.env.FAIL_PUBLISH === '1') process.exit(3);
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
  const bundleId = sha(await readFile(new URL('../SHA256SUMS', import.meta.url)));
  const cfg = {
    bundleId,
    artifactRoot: root,
    nginxRoot,
    mailbox,
    origin: 'https://admin.example.test',
    app: 'admin-v3',
    endpoint: '/_skrynia/',
    readCapFile: readCap,
    adminCapFile: adminCap,
    adminCapsFile: '',
    publishHelper: wrap,
    verifyHelper: wrap,
  };
  return {dir, cfg, root, nginxRoot, mailbox, bundleId};
}

test('install refuses a missing or invalid bundle identity and unsafe origins/endpoints', async t => {
  const {cfg} = await fixture(t);
  await assert.rejects(install({...cfg, bundleId: ''}), /missing or invalid out-of-band bootstrap bundle identity/);
  await assert.rejects(install({...cfg, bundleId: 'f'.repeat(40)}), /missing or invalid out-of-band bootstrap bundle identity/);
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

test('artifact root prerequisite is enforced, not a lexical optional comparison', async t => {
  const {dir, cfg, root, nginxRoot, mailbox} = await fixture(t);
  await assert.rejects(install({...cfg, artifactRoot: join(dir, 'missing-root')}), /existing real directory/);
  await chmod(root, 0o755);
  await assert.rejects(install(cfg), /permissions/);
  await chmod(root, 0o700);
  const rootAlias = join(dir, 'root-alias');
  await symlink(root, rootAlias);
  await assert.rejects(install({...cfg, artifactRoot: rootAlias}), /symlink/);
  await assert.rejects(install({...cfg, nginxRoot: dir}), /disjoint from nginx document root/);
  const mailboxAlias = join(dir, 'mailbox-alias');
  await symlink(mailbox, mailboxAlias);
  await assert.rejects(install({...cfg, mailbox: mailboxAlias}), /symlink/);
  assert.equal(nginxRoot !== mailbox, true);
});

test('nginx root and publication mailbox must be existing real directories', async t => {
  const {cfg, dir} = await fixture(t);
  await assert.rejects(install({...cfg, nginxRoot: join(dir, 'missing-nginx')}),
    /nginx document root must be an existing real directory/);
  const mailboxFile = join(dir, 'mailbox-as-file');
  await writeFile(mailboxFile, 'not a directory');
  await assert.rejects(install({...cfg, mailbox: mailboxFile}),
    /publication mailbox must be an existing real directory/);
});

test('install refuses to overwrite an existing private output', async t => {
  const {cfg} = await fixture(t);
  const first = await install(cfg);
  await assert.rejects(install(cfg), /EEXIST|exists/);
  assert.ok(first.release);
});

test('successful install persists a private recovery file and records the bundle identity', async t => {
  const {cfg, root, bundleId} = await fixture(t);
  const result = await install(cfg);
  assert.equal(result.passwordFile, join(root, 'apps', 'admin-v3', `password.${result.release}.txt`));
  const st = await stat(result.passwordFile);
  assert.ok(st.isFile() && !st.isSymbolicLink());
  assert.equal(st.mode & 0o777, 0o600);
  assert.equal(await readFile(result.passwordFile, 'utf8'), result.password + '\n');
  const marker = JSON.parse(await readFile(join(root, 'apps', 'admin-v3', '.installed'), 'utf8'));
  assert.equal(marker.bundleId, bundleId);
  assert.match(marker.bundleId, BUNDLE_ID_RE);
});

test('vault is scoped, roster is exact and published assets stay public-safe', async t => {
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
  const expectedRoster = [...WEB_ASSETS, 'unlock.html', 'bootstrap.vault.json'].sort(byRel);
  assert.deepEqual(manifest.files.map(f => f.rel), expectedRoster, 'manifest roster must be exact');
  const onDisk = (await readdir(releaseDir)).filter(n => /\.(js|html|css|json)$/.test(n)).sort(byRel);
  assert.deepEqual(onDisk, [...expectedRoster, 'manifest.json'].sort(byRel), 'published files must be exact');
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

test('unknown publish outcome fails closed without unpublish or recovery file', async t => {
  const {cfg, dir, root} = await fixture(t);
  const publishShell = join(dir, 'publish-fail.sh');
  await writeFile(publishShell, `#!/bin/sh\nexport FAIL_PUBLISH=1\nexec node '${join(dir, 'helper.mjs')}'\n`);
  await chmod(publishShell, 0o700);
  await assert.rejects(install({...cfg, publishHelper: publishShell}), /no unpublish attempted/);
  assert.equal(await readFile(join(dir, 'helper-actions.log'), 'utf8'), 'publish\n');
  const appRoot = await readdir(join(root, 'apps', 'admin-v3'));
  assert.equal(appRoot.some(n => n.startsWith('password.')), false, 'no recovery file on failed install');
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

test('SHA256SUMS covers exactly the published source roster', async () => {
  const sums = await readFile(new URL('../SHA256SUMS', import.meta.url), 'utf8');
  const listed = sums.trim().split('\n').map(l => l.split(/\s+/)[1]).sort();
  const expected = [
    'deploy.sh', 'install.mjs', 'native-publish.mjs', 'unlock.html', 'README.md',
    ...WEB_ASSETS.map(n => '../web-admin-v3/' + n),
  ].sort();
  assert.deepEqual(listed, expected, 'SHA256SUMS roster must be exact');
});

test('deploy.sh executes with verified SHA256SUMS anchor and exports bundle identity', async t => {
  const {spawnSync} = await import('node:child_process');
  const dir = await mkdtemp(join(tmpdir(), 'bootstrap-deploy-'));
  t.after(() => rm(dir, {recursive: true, force: true}));
  const bdir = join(dir, 'bootstrap');
  await mkdir(bdir, {recursive: true});
  // Minimal fake bundle: deploy.sh, stub install.mjs, and a SHA256SUMS that
  // covers exactly those files. The out-of-band anchor is the SHA256SUMS digest.
  const deploySrc = await readFile(new URL('../deploy.sh', import.meta.url));
  await writeFile(join(bdir, 'deploy.sh'), deploySrc, {mode: 0o700});
  await writeFile(join(bdir, 'install.mjs'), "process.stdout.write(JSON.stringify({bundleId:process.env.SKRYNIA_BOOTSTRAP_BUNDLE_ID||''})+'\\n');\n", {mode: 0o600});
  const sums = `${sha(deploySrc)}  deploy.sh\n${sha(await readFile(join(bdir, 'install.mjs')))}  install.mjs\n`;
  await writeFile(join(bdir, 'SHA256SUMS'), sums, {mode: 0o600});
  const anchor = sha(Buffer.from(sums));
  // Fake required inputs.
  const root = join(dir, 'private-root'), nginx = join(dir, 'nginx-root'), mailbox = join(dir, 'mailbox');
  for (const p of [root, nginx, mailbox]) { await mkdir(p, {mode: 0o700}); await chmod(p, 0o700); }
  for (const n of ['read.cap','admin.cap','pub.sh','ver.sh']) await writeFile(join(dir, n), n.endsWith('.cap') ? hex(n[0]) : '#!/bin/sh\nexit 0\n', {mode: 0o600});
  for (const n of ['pub.sh','ver.sh']) await chmod(join(dir, n), 0o700);
  const env = {
    ...process.env,
    SKRYNIA_BOOTSTRAP_SOURCE_SHA256: anchor,
    SKRYNIA_ARTIFACT_ROOT: root, SKRYNIA_NGINX_ROOT: nginx, SKRYNIA_PUBLICATION_MAILBOX: mailbox,
    SKRYNIA_ORIGIN_URL: 'https://admin.example.test',
    SKRYNIA_NATIVE_READ_CAP_FILE: join(dir, 'read.cap'), SKRYNIA_NATIVE_ADMIN_CAP_FILE: join(dir, 'admin.cap'),
    SKRYNIA_PUBLISH_HELPER: join(dir, 'pub.sh'), SKRYNIA_VERIFY_HELPER: join(dir, 'ver.sh'),
  };
  const cp = spawnSync('bash', [join(bdir, 'deploy.sh')], {env, encoding: 'utf8', timeout: 30000});
  assert.equal(cp.status, 0, cp.stderr || cp.stdout);
  const out = JSON.parse(cp.stdout.trim());
  assert.equal(out.bundleId, anchor);
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
