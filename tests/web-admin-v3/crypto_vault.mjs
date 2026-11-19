import assert from 'node:assert/strict';
import {decryptVault, encryptVault, digest} from '../../web-admin-v3/crypto.js';
import {
  createMemoryVaultStorage,
  createVaultSession,
  initializeVault,
  saveVaultSession,
  lockVaultSession,
} from '../../web-admin-v3/vault.js';

const password = 'correct horse battery staple secret';
const context = 'tools.a.vau.place/default';

{
  const value = {schema: 1, native: null, notes: ['ok'], nested: {x: 1}};
  const bytes = await encryptVault(value, password, context);
  const out = await decryptVault(bytes, password, context);
  assert.deepEqual(out, value);
  const text = new TextDecoder().decode(bytes);
  assert.equal(text.includes(password), false);
  assert.equal(text.includes('notes'), false);
  await assert.rejects(() => decryptVault(bytes, 'wrong-password', context), /Unlock failed/);
  await assert.rejects(() => decryptVault(bytes, password, 'other-context'), /Unlock failed/);
}

{
  const storage = createMemoryVaultStorage();
  const initial = {schema: 1, native: {endpoint: 'https://example.invalid/p0'}, pending: null};
  const session = await initializeVault(storage, password, context, initial);
  assert.equal(session.unresolved, false);
  const loaded = await createVaultSession(storage, password, context);
  assert.deepEqual(loaded.data, initial);
  const updated = {schema: 1, native: initial.native, pending: {op: 'x'}};
  await saveVaultSession(session, updated);
  assert.deepEqual(session.data, updated);
  const reloaded = await createVaultSession(storage, password, context);
  assert.deepEqual(reloaded.data, updated);
}

{
  const storage = createMemoryVaultStorage();
  const a = await initializeVault(storage, password, context, {schema: 1, a: 1});
  const b = await createVaultSession(storage, password, context);
  await saveVaultSession(a, {schema: 1, a: 2});
  await assert.rejects(() => saveVaultSession(b, {schema: 1, a: 3}), /modified elsewhere/);
  assert.equal(b.unresolved, false);
}

{
  const storage = createMemoryVaultStorage();
  const session = await initializeVault(storage, password, context, {schema: 1, a: 1});
  const badStorage = {
    async load() { return storage.load(); },
    async save() { throw Error('disk vanished'); },
  };
  session.storage = badStorage;
  await assert.rejects(() => saveVaultSession(session, {schema: 1, a: 2}), /disk vanished/);
  assert.equal(session.unresolved, true);
  await assert.rejects(() => saveVaultSession(session, {schema: 1, a: 3}), /outcome unknown/);
}


{
  const storage = createMemoryVaultStorage();
  const session = await initializeVault(storage, password, context, {schema: 1, a: 1});
  lockVaultSession(session);
  assert.equal(session.password, '');
  assert.equal(session.data, null);
  assert.equal(session.unresolved, true);
  await assert.rejects(() => saveVaultSession(session, {schema: 1, a: 2}), /outcome unknown/);
}

console.log('v3 crypto and encrypted browser vault tests passed');
