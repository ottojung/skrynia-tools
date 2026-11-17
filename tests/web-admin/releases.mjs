import assert from 'node:assert/strict';
import {createClient} from '../../web-admin/skrynia.js';
import {applications, history, activate, keys} from '../../web-admin/apps.js';
import {toBytes} from '../../web-admin/crypto.js';

const origin = process.env.SKRYNIA_TOOLS_ORIGIN;
const token = process.env.SKRYNIA_TOOLS_GRANT;
const client = createClient(origin + '/_skrynia');
const first = '1'.repeat(32);
const second = '2'.repeat(32);
const extra = '3'.repeat(32);
async function create(key, contents) {
  const data = typeof contents === 'string' ? contents : JSON.stringify(contents);
  const response = await client.send('POST', '/store/' + key,
    toBytes(data), token);
  assert.equal(response.status, 201, key);
}
async function add(name, id, commit) {
  const prefix = 'apps/' + name;
  await create(prefix + '/releases/' + id + '/index.html', '<!doctype html><html></html>');
  await create(prefix + '/versions/' + id + '.json', {
    schema:1, name, id, commit, files:[{path:'index.html',
      key: prefix + '/releases/' + id + '/index.html', bytes:27,
      sha256:'f'.repeat(64)}],
  });
  await create(prefix + '/versions/' + id + '.ready', 'ready');
}
await add('demo', first, 'older');
await add('demo', second, 'newer');
await add('sample', extra, 'sample');
const all = await applications(client, token);
assert.ok(all.includes('tools') && all.includes('demo') && all.includes('sample'));
const tools = await history(client, 'tools', token);
await assert.rejects(() => activate(client, 'tools', tools.active, token,
  'apps/tools/vault/' + 'f'.repeat(32) + '.json'), /different credential vault/);
let versions = await history(client, 'demo', token);
assert.deepEqual(versions.versions.map(v => v.id), [first, second]);
assert.equal(versions.active, null);
assert.equal(versions.versions[0].commit, 'older');
await activate(client, 'demo', first, token);
assert.equal((await history(client, 'demo', token)).active, first);
await activate(client, 'demo', second, token);
assert.equal((await history(client, 'demo', token)).active, second);
await activate(client, 'demo', first, token);
assert.equal((await history(client, 'demo', token)).active, first);
await activate(client, 'demo', second, token);
assert.equal((await history(client, 'demo', token)).active, second);
await assert.rejects(() => activate(client, 'demo', '0'.repeat(32), token),
  /404/);
const competitive = { ...client, async send(method, path, body, authority, headers) {
  if (method === 'PUT' && path === '/store/apps/demo/active.json') {
    await activate(client, 'demo', first, token);
    competitive.send = client.send;
  }
  return client.send(method, path, body, authority, headers);
}};
await assert.rejects(() => activate(competitive, 'demo', first, token),
  /conflict/);
const current = await history(client, 'demo', token);
assert.equal(current.active, first, 'contender must not overwrite the concurrent CAS');
await activate(client, 'demo', second, token);
const fabricated = {listPage: async (_scope, _token, after) =>
  after ? {keys:[after]}
    : {keys:['apps/test/active.json'],next:'apps/test/active.json'}};
await assert.rejects(() => keys(fabricated, 'apps/*', token),
  /Invalid Skrynia key listing/);
console.log('All app discovery, histories, initial activation, rollback, forward activation and CAS conflict passed');
