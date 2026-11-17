import {digest, randomId, toBytes} from './crypto.js';

const namePattern = /^[a-z0-9][a-z0-9-]{0,63}$/;
const idPattern = /^[0-9a-f]{32}$/;
const decoder = new TextDecoder();
function nameOk(name) {
  if (!namePattern.test(name)) throw Error('Invalid application name');
  return name;
}
function idOk(id) {
  if (!idPattern.test(id)) throw Error('Invalid release identifier');
  return id;
}
export async function keys(client, locations, token) {
  const result = [];
  let after = null;
  for (let page = 0; page < 100000; page++) {
    const batch = await client.listPage(locations, token, after);
    if (!Array.isArray(batch.keys) || batch.keys.length > 1000
      || batch.keys.some(key => typeof key !== 'string'
        || (after !== null && key <= after)))
      throw Error('Invalid Skrynia key listing');
    result.push(...batch.keys);
    if (batch.next == null) return result;
    if (!batch.keys.length || batch.next !== batch.keys.at(-1))
      throw Error('Invalid listing cursor; cannot claim complete results');
    after = batch.next;
  }
  throw Error('Listing exceeds 100,000 pages; cannot claim complete results');
}
export async function applications(client, token) {
  const found = new Set();
  for (const key of await keys(client, 'apps/*', token)) {
    const match = key.match(/^apps\/([a-z0-9][a-z0-9-]{0,63})\/(?:active\.json|versions\/[0-9a-f]{32}\.ready)$/);
    if (match) found.add(match[1]);
  }
  return [...found].sort();
}
async function releaseInfo(client, name, id) {
  const key = 'apps/' + name + '/versions/' + id + '.json';
  try {
    const bytes = await client.read(key);
    if (bytes.length > 4 * 1024 * 1024) throw Error('Manifest exceeds 4 MiB');
    const doc = JSON.parse(decoder.decode(bytes));
    if (doc.schema !== 1 || doc.id !== id || doc.name !== name
      || !Array.isArray(doc.files)) throw Error('Invalid release manifest');
    return {id, commit: typeof doc.commit === 'string' ? doc.commit : '',
      repo: typeof doc.repo === 'string' ? doc.repo : '',
      files: doc.files.length, usable: true};
  } catch (error) {
    return {id, usable: false, error: error.message};
  }
}
export async function history(client, name, token) {
  nameOk(name);
  const prefix = 'apps/' + name;
  const names = await keys(client, prefix + '/versions/*', token);
  const ready = names.map(key =>
    key.match(/^apps\/[a-z0-9-]+\/versions\/([0-9a-f]{32})\.ready$/))
    .filter(Boolean).map(match => match[1]);
  const versions = [];
  for (let i = 0; i < ready.length; i += 8) {
    const chunk = await Promise.all(ready.slice(i, i + 8)
      .map(id => releaseInfo(client, name, id)));
    versions.push(...chunk);
  }
  let active = null;
  try {
    const value = await client.read(prefix + '/active.json');
    const pointer = JSON.parse(decoder.decode(value));
    if (pointer.schema !== 1 || !idPattern.test(pointer.release))
      throw Error('Invalid active release pointer');
    active = pointer.release;
  } catch (error) {
    if (error.status !== 404) throw error;
  }
  return {name, active, versions};
}
export async function activate(client, name, id, token, adminVault = null) {
  nameOk(name);
  idOk(id);
  const prefix = 'apps/' + name;
  const marker = await client.read(prefix + '/versions/' + id + '.ready');
  if (decoder.decode(marker) !== 'ready')
    throw Error('Selected release is not finalized');
  const info = await releaseInfo(client, name, id);
  if (!info.usable) throw Error('Cannot activate release: ' + info.error);
  if (adminVault?.startsWith(prefix + '/vault/')) {
    const configBytes = await client.read(prefix
      + '/releases/' + id + '/config.json');
    const config = JSON.parse(decoder.decode(configBytes));
    if (config.schema !== 1 || config.vault !== adminVault)
      throw Error('Admin release has a different credential vault; activation blocked');
  }
  const key = prefix + '/active.json';
  let old = null;
  try {
    old = await client.read(key);
    const previous = JSON.parse(decoder.decode(old));
    if (previous.schema !== 1 || !idPattern.test(previous.release))
      throw Error('Invalid current release pointer');
  } catch (error) {
    if (error.status !== 404) throw error;
  }
  const data = toBytes(JSON.stringify({schema: 1, release: id, nonce: randomId()}));
  const method = old ? 'PUT' : 'POST';
  const headers = old ? {'X-Skrynia-If-SHA256': await digest(old)} : {};
  const response = await client.send(method,
    '/store/' + key, data, token, headers);
  if (response.status === 412 || response.status === 409)
    throw Error('Activation conflict: active release changed concurrently');
  if (response.status !== (old ? 200 : 201))
    throw Error('Activation outcome not confirmed (HTTP ' + response.status
      + '); refresh the active pointer before retrying');
  return id;
}
