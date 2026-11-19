import {decryptVault, encryptVault, digest} from './crypto.js';

export function createMemoryVaultStorage(initialBytes = null) {
  let bytes = initialBytes ? new Uint8Array(initialBytes) : null;
  let currentDigest = null;
  return {
    kind: 'memory',
    async load() {
      if (bytes == null) throw Object.assign(Error('Vault not found'), {code: 'not_found'});
      if (currentDigest == null) currentDigest = await digest(bytes);
      return {bytes: new Uint8Array(bytes), digest: currentDigest};
    },
    async save(nextBytes, expectedDigest) {
      const next = new Uint8Array(nextBytes);
      if (bytes == null) {
        if (expectedDigest != null) {
          throw Object.assign(Error('Vault does not exist'), {code: 'conflict'});
        }
      } else {
        if (currentDigest == null) currentDigest = await digest(bytes);
        if (expectedDigest !== currentDigest) {
          throw Object.assign(Error('Vault was modified elsewhere'), {code: 'conflict'});
        }
      }
      const nextDigest = await digest(next);
      bytes = next;
      currentDigest = nextDigest;
      return {digest: nextDigest};
    },
  };
}

export async function createVaultSession(storage, password, context) {
  const loaded = await storage.load();
  const data = await decryptVault(loaded.bytes, password, context);
  return {
    storage,
    context,
    password,
    data,
    hash: loaded.digest,
    unresolved: false,
  };
}

export async function initializeVault(storage, password, context, initialData) {
  if (!initialData || initialData.schema !== 1) {
    throw Error('Invalid initial vault data');
  }
  const bytes = await encryptVault(initialData, password, context);
  const saved = await storage.save(bytes, null);
  return {
    storage,
    context,
    password,
    data: structuredClone(initialData),
    hash: saved.digest,
    unresolved: false,
  };
}

export async function saveVaultSession(session, updated) {
  if (session.unresolved) {
    throw Error('Vault save outcome unknown; reload to reconcile');
  }
  if (!updated || updated.schema !== 1) throw Error('Invalid vault contents');
  const bytes = await encryptVault(updated, session.password, session.context);
  try {
    const saved = await session.storage.save(bytes, session.hash);
    session.hash = saved.digest;
    session.data = structuredClone(updated);
    return saved;
  } catch (error) {
    if (error && error.code === 'conflict') throw error;
    session.unresolved = true;
    throw error;
  }
}

export function lockVaultSession(session) {
  if (!session) return;
  session.password = '';
  session.data = null;
  session.hash = null;
}
