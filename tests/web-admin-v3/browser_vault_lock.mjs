import assert from 'node:assert/strict';
import {createLocalStorageVaultStorage} from '../../web-admin-v3/browser_storage.js';

const previousStorage = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
const previousNavigator = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
function replace(name, value) {
  Object.defineProperty(globalThis, name, {value, configurable: true, writable: true});
}
function restore(name, descriptor) {
  if (descriptor) Object.defineProperty(globalThis, name, descriptor);
  else delete globalThis[name];
}

try {
  const values = new Map();
  replace('localStorage', {
    getItem: key => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  });

  replace('navigator', {});
  assert.throws(() => createLocalStorageVaultStorage('test'), /Web Locks/);

  let tail = Promise.resolve();
  let active = 0;
  replace('navigator', {locks: {
    request: (name, opts, callback) => {
      assert.equal(name, 'skrynia-v3-vault:test');
      assert.equal(opts.mode, 'exclusive');
      const result = tail.then(async () => {
        active++;
        assert.equal(active, 1, 'writes must not overlap');
        try { return await callback(); }
        finally { active--; }
      });
      tail = result.catch(() => {});
      return result;
    }
  }});

  const first = createLocalStorageVaultStorage('test');
  const second = createLocalStorageVaultStorage('test');
  const a = Uint8Array.from([1, 2, 3]);
  const b = Uint8Array.from([4, 5, 6]);
  const c = Uint8Array.from([7, 8, 9]);
  const created = await first.save(a, null);
  assert.deepEqual((await second.load()).bytes, a);

  const writes = await Promise.allSettled([
    first.save(b, created.digest),
    second.save(c, created.digest),
  ]);
  assert.equal(writes.filter(x => x.status === 'fulfilled').length, 1);
  const conflict = writes.find(x => x.status === 'rejected');
  assert.equal(conflict.reason.code, 'conflict');
  assert.deepEqual((await first.load()).bytes, b);
  assert.deepEqual((await second.load()).bytes, b);
  console.log('v3 vault atomic cross-tab lock and stale-session rejection tests passed');
} finally {
  restore('localStorage', previousStorage);
  restore('navigator', previousNavigator);
}
