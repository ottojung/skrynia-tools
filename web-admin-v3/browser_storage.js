import {digest} from './crypto.js';

export function createLocalStorageVaultStorage(key = 'skrynia-v3-admin-vault') {
  const storage = globalThis.localStorage;
  if (!storage) throw Error('Local browser storage is unavailable');
  return {
    kind: 'localStorage',
    async load() {
      const raw = storage.getItem(key);
      if (raw == null) throw Object.assign(Error('Vault not found'), {code: 'not_found'});
      const bytes = Uint8Array.from(atob(raw), c => c.charCodeAt(0));
      return {bytes, digest: await digest(bytes)};
    },
    async save(bytes, expectedDigest) {
      const next = new Uint8Array(bytes);
      const existing = storage.getItem(key);
      if (existing == null) {
        if (expectedDigest != null) {
          throw Object.assign(Error('Vault does not exist'), {code: 'conflict'});
        }
      } else {
        const existingBytes = Uint8Array.from(atob(existing), c => c.charCodeAt(0));
        const current = await digest(existingBytes);
        if (expectedDigest !== current) {
          throw Object.assign(Error('Vault was modified elsewhere'), {code: 'conflict'});
        }
      }
      let raw = '';
      for (let i = 0; i < next.length; i += 8192) {
        raw += String.fromCharCode(...next.subarray(i, i + 8192));
      }
      const encoded = btoa(raw);
      storage.setItem(key, encoded);
      return {digest: await digest(next)};
    },
  };
}
