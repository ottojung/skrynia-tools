import {digest} from './crypto.js';
export function validateConfig(config) {
  if (!config || config.schema !== 1
    || !/^apps\/[a-z0-9-]+\/vault\/[0-9a-f]{32}\.json$/.test(config.vault))
    throw Error('Invalid admin installation');
  return config;
}
export function createClient(api = '/_skrynia', transport = fetch) {
  const url = path => api.replace(/\/$/, '') + path;
  async function send(method, path, body, token, headers = {}) {
    let response;
    try {
      response = await transport(url(path), {method, cache: 'no-store',
        redirect: 'error', headers: {
          ...(body === undefined ? {} : {'Content-Type': 'application/octet-stream'}),
          ...(token ? {Authorization: 'Bearer ' + token} : {}), ...headers},
        ...(body === undefined ? {} : {body})});
    } catch {
      throw Error(method === 'GET' ? 'Read failed; check connectivity'
        : 'Mutation outcome UNKNOWN. Do not automatically retry');
    }
    return response;
  }
  async function json(method, path, body, token) {
    const response = await send(method, path,
      body === undefined ? undefined : JSON.stringify(body), token,
      body === undefined ? {} : {'Content-Type':'application/json'});
    if (response.status !== 200)
      throw Error(method + ' failed (HTTP ' + response.status + ')');
    return response.json();
  }
  const storePath = key => '/store/' + key.split('/').map(encodeURIComponent).join('/');
  async function read(key) {
    const response = await send('GET', storePath(key));
    if (response.status !== 200) {
      const error = Error('Object read failed (HTTP ' + response.status + ')');
      error.status = response.status;
      throw error;
    }
    return new Uint8Array(await response.arrayBuffer());
  }
  async function listPage(locations, token, after = null) {
    const params = new URLSearchParams({locations, limit:'1000'});
    if (after !== null) params.set('after', after);
    const response = await send('GET', '/store?' + params, undefined, token);
    if (response.status !== 200)
      throw Error('Listing rejected (HTTP ' + response.status + ')');
    return response.json();
  }
  async function replace(key, bytes, token, expectedHash) {
    const method = expectedHash ? 'PUT' : 'POST';
    const response = await send(method, storePath(key), bytes, token,
      expectedHash ? {'X-Skrynia-If-SHA256': expectedHash} : {});
    if (response.status === 412 || response.status === 409)
      throw Error('Vault was modified elsewhere. Reload and unlock again');
    if (response.status !== (expectedHash ? 200 : 201))
      throw Error('Vault update rejected (HTTP ' + response.status + ')');
    return digest(bytes);
  }
  return {read, replace, json, send, listPage};
}
export async function openVault(client, key) {
  const bytes = await client.read(key);
  return {bytes, hash: await digest(bytes)};
}
export async function saveVault(session, updated) {
  const {encrypt} = await import('./crypto.js');
  const bytes = await encrypt(updated, session.password, session.key);
  try {
    const hash = await session.client.replace(
      session.key, bytes, session.data.parentToken, session.hash);
    session.hash = hash;
    session.data = updated;
  } catch (error) {
    session.unresolved = true;
    throw error;
  }
}
export function validateGrant(spec) {
  const validKey = value => value.split('/').every(part =>
    /^[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*$/.test(part));
  const prefix = spec.locations.endsWith('*')
    ? spec.locations.slice(0, -1).replace(/\/$/, '') : spec.locations;
  if (!spec.locations || spec.locations.length > 257
    || (spec.locations !== '*' && !validKey(prefix)))
    throw Error('Invalid location matcher');
  if (!Array.isArray(spec.operations) || spec.operations.length === 0
    || spec.operations.some(op => !['create','modify','delete','list',
      'delegate','subscribe'].includes(op))
    || new Set(spec.operations).size !== spec.operations.length)
    throw Error('Select unique valid grant operations');
  for (const name of ['maxBytes', 'maxObjects']) {
    if (!Number.isSafeInteger(spec[name]) || spec[name] < 0)
      throw Error(name + ' must be a non-negative safe integer');
  }
  return spec;
}
