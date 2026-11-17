import {createHash} from 'node:crypto';
export const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export function createTransport(base, token, transport = fetch) {
  const endpoint = base.replace(/\/$/, '') + '/store/';
  const validate = key => {
    if (!key.split('/').every(part => /^[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*$/.test(part)))
      throw Error('Invalid object key: ' + key);
    return endpoint + key;
  };
  async function request(method, key, bytes, expected) {
    let response;
    try {
      response = await transport(validate(key), {method, redirect: 'error',
        cache: 'no-store', headers: {
          ...(method !== 'GET' ? {Authorization: 'Bearer ' + token} : {}),
          ...(expected ? {'X-Skrynia-If-SHA256': expected} : {})},
        ...(bytes === undefined ? {} : {body:bytes})});
    } catch {
      throw Error(method + ' outcome UNKNOWN for ' + key
        + '; do not automatically retry');
    }
    return response;
  }
  async function get(key) {
    const response = await request('GET', key);
    if (response.status === 404) return null;
    if (response.status !== 200)
      throw Error('Read rejected (HTTP ' + response.status + ')');
    return Buffer.from(await response.arrayBuffer());
  }
  async function create(key, data) {
    const response = await request('POST', key, data);
    if (response.status === 201) return true;
    if (response.status === 409) return false;
    throw Error('Create rejected (HTTP ' + response.status + ')');
  }
  async function createExact(key, data) {
    if (await create(key, data)) return;
    const old = await get(key);
    if (!old?.equals(data)) throw Error('Immutable object collision: ' + key);
  }
  async function stable(key, data) {
    if (await create(key, data)) return;
    const old = await get(key);
    if (!old) throw Error('Stable object disappeared');
    if (old.equals(data)) return;
    const response = await request('PUT', key, data, hash(old));
    if (response.status !== 200)
      throw Error('Stable object change rejected (HTTP '
        + response.status + '); reconcile before retry');
  }
  async function activate(key, data) {
    const previous = await get(key);
    const response = previous
      ? await request('PUT', key, data, hash(previous))
      : await request('POST', key, data);
    if (response.status !== (previous ? 200 : 201))
      throw Error('Activation not confirmed (HTTP ' + response.status + ')');
  }
  return {get, create, createExact, stable, activate};
}
