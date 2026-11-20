import {test} from 'node:test';
import assert from 'node:assert/strict';
import {randomBytes, createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {mkdtemp, mkdir, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import net from 'node:net';
import {createNativeClient, NativeRejected} from '../../native-client/index.mjs';
const bin = process.env.SKRYNIA_V3_SERVER_BIN;
const wasm = process.env.SKRYNIA_V3_PROGRAM0_WASM;
const enabled = Boolean(bin && wasm);
const utf = x => new TextEncoder().encode(x);
const hash = x => createHash('sha256').update(x).digest();
async function port() {
  const s = net.createServer();
  await new Promise(ok => s.listen(0, '127.0.0.1', ok));
  const n = s.address().port;
  await new Promise(ok => s.close(ok));
  return n;
}
test('real native kernel: installation, versions, restart, CAS data', {skip: !enabled}, async () => {
  const dir = await mkdtemp(join(tmpdir(), 'skrynia-native-js-'));
  const dataDir = join(dir, 'state');
  const n = await port();
  const native = createNativeClient('http://127.0.0.1:' + n + '/');
  let process;
  const start = async () => {
    process = spawn(bin, [], {env: {...globalThis.process.env,
      SKRYNIA_BIND: '127.0.0.1:' + n, SKRYNIA_ROOT_WASM: wasm,
      SKRYNIA_DATA_DIR: dataDir}, stdio: 'ignore'});
    for (let i = 0; i < 120; i++) {
      if (process.exitCode !== null) throw Error('v3 server exited on startup');
      try { await native.programs(); return; } catch {}
      await new Promise(r => setTimeout(r, 100));
    }
    throw Error('v3 server did not become ready');
  };
  const stop = async () => {
    if (process.exitCode === null) process.kill('SIGTERM');
    await Promise.race([new Promise(ok => process.once('exit', ok)),
      new Promise(ok => setTimeout(ok, 1500))]);
    if (process.exitCode === null) process.kill('SIGKILL');
  };
  try {
    await mkdir(join(dataDir, 'programs'), {recursive: true});
    await writeFile(join(dataDir, 'programs/operator.cap'), new Uint8Array(32).fill(85),
      {mode: 0o600, flag: 'wx'});
    await start();
    const admin = randomBytes(32), secret = new Uint8Array(32).fill(85);
    await assert.rejects(native.register(randomBytes(32), admin, utf('one')), NativeRejected);
    const id = await native.register(secret, admin, utf('one'));
    assert.deepEqual(Buffer.from(id), hash(admin));
    assert.equal((await native.current(id)).revision, 1n);
    const h2 = await native.stage(admin, utf('two'));
    await assert.rejects(native.activate(admin, 1n, h2), NativeRejected);
    await native.seal(admin, h2);
    assert.equal((await native.activate(admin, 1n, h2)).revision, 2n);
    assert.equal((await native.versions(id)).length, 2);
    assert.equal((await native.programs()).length, 1);
    await stop(); await start();
    assert.equal((await native.current(id)).revision, 2n);
    const h1 = hash(utf('one'));
    assert.equal((await native.activate(admin, 2n, h1)).revision, 3n);
    await assert.rejects(native.activate(admin, 1n, h2), NativeRejected);
    const wc = randomBytes(32), rc = hash(wc), key = utf('native-key');
    const value = utf('first-value');
    const sha = await native.appPut(wc, key, null, value);
    assert.deepEqual(Buffer.from(sha), hash(value));
    assert.deepEqual(await native.appGet(rc, key), value);
    const updated = utf('updated-value');
    await native.appPut(wc, key, sha, updated);
    await assert.rejects(native.appPut(wc, key, sha, updated), NativeRejected);
    assert.deepEqual(await native.appGet(rc, key), updated);
  } finally {
    if (process && process.exitCode === null) await stop();
    await rm(dir, {recursive: true, force: true});
  }
});
