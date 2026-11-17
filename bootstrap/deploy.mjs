import {readFile} from 'node:fs/promises';
import {join} from 'node:path';
import {randomBytes as nodeRandom} from 'node:crypto';
import {encrypt, decrypt, randomId} from '../web-admin/crypto.js';
import {createTransport, hash} from './transport.mjs';

const api = process.env.SKRYNIA_URL?.replace(/\/$/, '');
const parent = process.env.SKRYNIA_TOOLS_GRANT;
const rawOrigin = process.env.SKRYNIA_TOOLS_ORIGIN;
const source = process.argv[2];
const app = process.env.SKRYNIA_TOOLS_APP || 'tools';
if (!api || !parent || !rawOrigin || !source)
  throw Error('Missing Skrynia URL, user grant, app origin or source directory');
if (!/^[a-z0-9][a-z0-9-]{0,63}$/.test(app))
  throw Error('Invalid admin application name');
function validatedUrl(value, originOnly = false) {
  const parsed = new URL(value);
  if (parsed.username || parsed.password || parsed.search || parsed.hash
    || (originOnly && parsed.pathname !== '/')
    || (!originOnly && parsed.pathname === '/'))
    throw Error('Unexpected URL shape');
  if (parsed.protocol !== 'https:'
    && !(parsed.protocol === 'http:' &&
      ['127.0.0.1', 'localhost', '[::1]'].includes(parsed.hostname)))
    throw Error('HTTPS required outside local development');
  return parsed;
}
validatedUrl(api);
const origin = validatedUrl(rawOrigin.replace(/\/?$/, '/'), true).origin;
if (new URL(api).origin === origin)
  throw Error('Admin interface requires a separate browser origin');
const ready = await fetch(origin + '/_skrynia/health', {redirect: 'error'});
if (ready.status !== 200)
  throw Error('Admin origin does not proxy the Skrynia API (HTTP '
    + ready.status + ')');
const deploy = createTransport(api, parent);
const prefix = 'apps/' + app;
const before = await deploy.get(prefix + '/active.json');
let password;
let vaultKey;
if (before) {
  const pointer = JSON.parse(before.toString('utf8'));
  if (!/^[0-9a-f]{32}$/.test(pointer.release))
    throw Error('Existing admin release has an invalid pointer');
  const previous = await deploy.get(prefix + '/releases/'
    + pointer.release + '/config.json');
  if (!previous) throw Error('Existing admin configuration is missing');
  const config = JSON.parse(previous.toString('utf8'));
  if (!new RegExp('^' + prefix + '/vault/[a-f0-9]{32}\\.json$')
    .test(config.vault)) throw Error('Existing admin vault path is invalid');
  vaultKey = config.vault;
  password = process.env.SKRYNIA_TOOLS_ACCESS_SECRET;
  if (!password) throw Error('Set SKRYNIA_TOOLS_ACCESS_SECRET for an upgrade');
  const contents = await deploy.get(vaultKey);
  if (!contents) throw Error('Existing admin vault is missing');
  const old = await decrypt(contents, password, vaultKey);
  if (old.parentToken !== parent)
    throw Error('Parent grant does not match the existing vault');
} else {
  vaultKey = prefix + '/vault/' + randomId() + '.json';
  password = nodeRandom(32).toString('base64url');
  const state = {schema: 1, parentToken: parent, grants: [], pending: null};
  const ciphertext = Buffer.from(await encrypt(state, password, vaultKey));
  await deploy.createExact(vaultKey, ciphertext);
}
const release = randomId();
const files = ['index.html', 'styles.css', 'crypto.js',
  'skrynia.js', 'admin.js', 'apps.js', 'apps-ui.js', 'main.js'];
const config = Buffer.from(JSON.stringify({schema: 1, vault: vaultKey}));
const assets = [...await Promise.all(files.map(async path => [
  path, Buffer.from(await readFile(join(source, 'web-admin', path)))])),
['config.json', config]];
const manifestFiles = [];
for (const [path, data] of assets) {
  const key = prefix + '/releases/' + release + '/' + path;
  await deploy.createExact(key, data);
  manifestFiles.push({path, key, sha256: hash(data), bytes: data.length});
}
const record = {schema: 1, name: app, id: release, commit:
  process.env.SKRYNIA_TOOLS_SOURCE_COMMIT || '',
  repo: 'https://github.com/ottojung/skrynia-tools',
  subdir: 'web-admin', files: manifestFiles};
await deploy.createExact(prefix + '/versions/' + release + '.json',
  Buffer.from(JSON.stringify(record)));
await deploy.stable(prefix + '/index.html',
  Buffer.from(await readFile(join(source, 'src/http_app/landing.html'))));
await deploy.createExact(prefix + '/versions/' + release + '.ready',
  Buffer.from('ready'));
await deploy.activate(prefix + '/active.json',
  Buffer.from(JSON.stringify({schema: 1, release, nonce: randomId()})));
const browserUrl = origin + '/';
const page = await fetch(origin + '/releases/' + release + '/index.html',
  {redirect: 'error'});
if (page.status !== 200
  || page.headers.get('content-security-policy')?.includes('sandbox'))
  throw Error('Published, but app origin is not executable. Save the vault key: '
    + vaultKey + ' and verify gateway settings');
console.log('');
console.log('Skrynia Tools web administration published.');
console.log('Admin URL: ' + browserUrl);
console.log('Access secret: ' + password);
console.log('Store this secret privately; it cannot be recovered from Skrynia.');
console.log('Vault is encrypted, stored at: ' + vaultKey);
