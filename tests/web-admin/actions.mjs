import assert from 'node:assert/strict';
import {mkdtemp, mkdir, readFile, writeFile, symlink, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {createClient} from '../../web-admin/skrynia.js';
import {history, activate} from '../../web-admin/apps.js';

const base = process.env.SKRYNIA_URL;
const token = process.env.SKRYNIA_TOOLS_GRANT;
const prefix = 'apps/actiondemo';
const working = await mkdtemp(join(tmpdir(), 'skrynia-web-action-'));
const directory = join(working, 'app');
const outputs = join(working, 'outputs.txt');
const client = createClient(base);
await mkdir(join(directory,'assets'), {recursive:true});
const html = '<!doctype html><html><script src="./assets/app.js"></script></html>';
const commit = 'a'.repeat(40);
const run = (script, extra = {}) => {
  const result = spawnSync(process.execPath, [script], {
    cwd: process.cwd(),
    encoding:'utf8',
    env:{...process.env,
      GITHUB_REPOSITORY:'ottojung/sample',
      SKRYNIA_APP_NAME:'actiondemo',
      SKRYNIA_BUILT_COMMIT:commit,
      SKRYNIA_BUILT_DIRECTORY:directory,
      GITHUB_OUTPUT:outputs,
      SKRYNIA_TOKEN:token,
      ...extra},
  });
  if (result.status !== 0) throw Error(
    'Action failed: ' + result.stderr + '\n' + result.stdout);
  return result.stdout;
};
try {
  await writeFile(join(directory, 'index.html'), html);
  await writeFile(join(directory,'assets','app.js'), 'console.log("v1");');
  run('action/deploy/upload.mjs');
  const first = (await history(client, 'actiondemo', token)).active;
  assert.match(first, /^[0-9a-f]{32}$/);
  const release = await client.read(prefix+'/releases/'+first+'/assets/app.js');
  assert.equal(new TextDecoder().decode(release), 'console.log("v1");');
  await writeFile(join(directory,'assets','app.js'), 'console.log("v2");');
  run('action/deploy/upload.mjs');
  const data = await history(client, 'actiondemo', token);
  const second = data.active;
  assert.notEqual(first, second);
  assert.equal(data.versions.length, 2);
  assert.ok(data.versions.every(item => item.usable));
  const stable = join(working, 'sw.js');
  await writeFile(stable, 'self.addEventListener("push",()=>{});');
  run('action/publish-stable/upload.mjs', {
    SKRYNIA_STABLE_ASSET:'sw.js', SKRYNIA_STABLE_FILE:stable,
  });
  run('action/publish-stable/upload.mjs', {
    SKRYNIA_STABLE_ASSET:'sw.js', SKRYNIA_STABLE_FILE:stable,
  });
  await activate(client, 'actiondemo', first, token);
  assert.equal((await history(client, 'actiondemo', token)).active, first);
  const worker = await client.read(prefix+'/sw.js');
  assert.equal(new TextDecoder().decode(worker),
    'self.addEventListener("push",()=>{});');
  await activate(client, 'actiondemo', second, token);
  assert.equal((await history(client,'actiondemo',token)).active,second);
  const bogus = spawnSync(process.execPath,
    ['action/publish-stable/upload.mjs'], {
      cwd:process.cwd(),encoding:'utf8', env:{
        ...process.env, SKRYNIA_APP_NAME:'actiondemo',SKRYNIA_TOKEN:token,
        SKRYNIA_STABLE_ASSET:'../active.json',SKRYNIA_STABLE_FILE:stable},
    });
  assert.notEqual(bogus.status,0,'Must refuse stable-asset path traversal');
  await symlink(join(directory,'index.html'), join(directory,'assets','link.js'));
  const blocked = spawnSync(process.execPath,['action/deploy/upload.mjs'],{
    cwd:process.cwd(),encoding:'utf8',env:{
      ...process.env,GITHUB_REPOSITORY:'ottojung/sample',
      SKRYNIA_APP_NAME:'actiondemo',SKRYNIA_BUILT_COMMIT:commit,
      SKRYNIA_BUILT_DIRECTORY:directory,GITHUB_OUTPUT:outputs,
      SKRYNIA_TOKEN:token},
  });
  assert.notEqual(blocked.status,0,'Must refuse release symlink');
  const output = await readFile(outputs,'utf8');
  assert.equal((output.match(/^release=/gm) || []).length,2);
  console.log('GitHub Actions: direct HTTP release publishing, stable worker, rollback, forward activation and unsafe-path rejection passed');
} finally {
  await rm(working,{recursive:true,force:true});
}
