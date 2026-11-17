import {readFile, readdir, lstat, appendFile} from 'node:fs/promises';
import {join, resolve} from 'node:path';
import {randomBytes} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {createTransport, hash} from '../../bootstrap/transport.mjs';

const env = process.env;
const required = name => {
  const value = env[name];
  if (!value) throw Error('Required GitHub Action input missing: ' + name);
  return value;
};
const app = required('SKRYNIA_APP_NAME');
const commit = required('SKRYNIA_BUILT_COMMIT');
const source = resolve(required('SKRYNIA_BUILT_DIRECTORY'));
const url = required('SKRYNIA_URL');
const token = required('SKRYNIA_TOKEN');
const outputs = required('GITHUB_OUTPUT');
if (!/^[a-z0-9][a-z0-9-]{0,63}$/.test(app))
  throw Error('Invalid application name');
if (!/^[0-9a-f]{40}$/.test(commit)) throw Error('Invalid source commit');
const keyPart = /^[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*$/;
async function scan(folder, prefix = '') {
  const entries = await readdir(folder, {withFileTypes:true});
  const result = [];
  for (const entry of entries) {
    if (!keyPart.test(entry.name)) throw Error('Invalid artifact name');
    const path = join(folder, entry.name);
    const name = prefix ? prefix + '/' + entry.name : entry.name;
    const metadata = await lstat(path);
    if (metadata.isSymbolicLink()) throw Error('Symlinks forbidden in releases');
    if (metadata.isDirectory()) result.push(...await scan(path, name));
    else if (metadata.isFile()) result.push([name, path]);
    else throw Error('Nonregular file in release');
    if (result.length > 100000) throw Error('Too many release assets');
  }
  return result;
}
if (!(await lstat(source)).isDirectory()) throw Error('Invalid release directory');
const files = (await scan(source)).sort((a,b) => a[0].localeCompare(b[0]));
if (!files.some(([name]) => name === 'index.html'))
  throw Error('Application release must contain index.html');
const transport = createTransport(url, token);
const id = randomBytes(16).toString('hex');
const prefix = 'apps/' + app;
const manifest = [];
for (const [path, sourceFile] of files) {
  const bytes = await readFile(sourceFile);
  const key = prefix + '/releases/' + id + '/' + path;
  await transport.createExact(key, bytes);
  manifest.push({path, key, sha256:hash(bytes), bytes:bytes.length});
}
const record = {schema:1, name:app, id, commit,
  repo:env.SKRYNIA_SOURCE_REPO || 'https://github.com/' + required('GITHUB_REPOSITORY'),
  subdir:env.SKRYNIA_APP_SUBDIR || '.', files:manifest};
await transport.createExact(prefix + '/versions/' + id + '.json',
  Buffer.from(JSON.stringify(record)));
const here = fileURLToPath(new URL('../../src/http_app/landing.html', import.meta.url));
await transport.stable(prefix + '/index.html', await readFile(here));
await transport.createExact(prefix + '/versions/' + id + '.ready',
  Buffer.from('ready'));
await transport.activate(prefix + '/active.json',
  Buffer.from(JSON.stringify({schema:1,release:id,nonce:randomBytes(16).toString('hex')})));
const browserUrl = url.replace(/\/$/, '') + '/store/' + prefix + '/index.html';
await appendFile(outputs, 'release=' + id + '\ncommit=' + commit
  + '\nurl=' + browserUrl + '\n');
console.log('Published app', app, 'release', id);
