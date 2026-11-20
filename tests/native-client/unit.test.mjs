import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createNativeClient, request, decode, active, listing, hex, unhex,
  NativeOutcomeUnknown, NativeRejected, NativeTransportError} from '../../native-client/index.mjs';
const cap = n => new Uint8Array(32).fill(n);
function response(status, value = new Uint8Array()) {
  const body = new Uint8Array(6 + value.length);
  body[0] = 1; body[1] = status;
  new DataView(body.buffer).setUint32(2, value.length, true);
  body.set(value, 6);
  return new Response(body, {status: 200});
}
function pointer(hash, revision) {
  const b = new Uint8Array(42);
  b[0] = 3; b[1] = 1; b.set(hash, 2);
  new DataView(b.buffer).setBigUint64(34, BigInt(revision), true);
  return b;
}
test('authenticated native registration is opcode 15, legacy 1 retired', () => {
  const b = request.register(cap(85), cap(10), Uint8Array.of(4, 5));
  assert.deepEqual([...b.slice(0, 4)], [1, 15, 32, 0]);
  assert.deepEqual([...b.slice(4, 36)], [...cap(85)]);
  assert.deepEqual([...b.slice(36, 40)], [32, 0, 10, 10]);
  assert.equal(b.byteLength, 2 + 34 + 34 + 4 + 2);
});
test('pointer and history parsing preserves u64 revisions and cursor bytes', () => {
  const a = active(pointer(cap(6), 18446744073709551615n));
  assert.equal(a.revision, 18446744073709551615n);
  assert.deepEqual(a.hash, cap(6));
  const history = new Uint8Array(66);
  history[0] = 2; history.set(cap(1), 2); history.set(cap(2), 34);
  assert.deepEqual(listing(history), [cap(1), cap(2)]);
  const page = request.listVersions(cap(3), cap(4), 5);
  assert.deepEqual([...page.slice(0, 4)], [1, 13, 32, 0]);
  assert.deepEqual([...page.slice(36, 39)], [1, 4, 4]);
  assert.deepEqual([...page.slice(-2)], [5, 0]);
});
test('invalid frame, active pointer, cursor and caps fail without I/O', () => {
  assert.throws(() => request.register(cap(1).slice(1), cap(2), cap(3)), /32 bytes/);
  assert.throws(() => request.activateCode(cap(1), -1, cap(2)), /u64/);
  assert.throws(() => request.listPrograms(null, 257), /1..256/);
  assert.throws(() => request.appPut(cap(1), cap(2), cap(3).slice(1), cap(4)), /32 bytes/);
  assert.throws(() => decode(Uint8Array.of(1, 0, 9, 0, 0, 0)), /length/);
  assert.throws(() => active(new Uint8Array(42)), /schema/);
  assert.throws(() => listing(Uint8Array.of(2, 0, 1, 2, 3)), /list/);
  assert.deepEqual(unhex(hex(cap(7))), cap(7));
});
test('transport never forwards secrets through URL, cookies or redirects', async () => {
  let opts;
  const native = createNativeClient('http://127.0.0.1:12345/', async (url, init) => {
    assert.equal(url, 'http://127.0.0.1:12345/');
    opts = init; return response(0, cap(3));
  });
  assert.deepEqual(await native.register(cap(85), cap(2), Uint8Array.of(5)), cap(3));
  assert.equal(opts.method, 'POST');
  assert.equal(opts.redirect, 'error');
  assert.equal(opts.credentials, 'omit');
  assert.equal(opts.referrerPolicy, 'no-referrer');
  assert.equal(opts.cache, 'no-store');
  assert.throws(() => createNativeClient('http://unsafe.example/'), /HTTPS/);
  assert.throws(() => createNativeClient('https://u:p@example.com/'), /credentials/);
  assert.throws(() => createNativeClient('https://example.com/?token=x'), /credentials/);
});
test('failed mutation transport or malformed reply is UNKNOWN, never auto-retried', async () => {
  let count = 0;
  const broken = createNativeClient('http://localhost:123/', async () => {
    count++; throw Error('disconnected');
  });
  await assert.rejects(broken.stage(cap(5), Uint8Array.of(1)), NativeOutcomeUnknown);
  assert.equal(count, 1);
  await assert.rejects(broken.current(cap(3)), NativeTransportError);
  const serverError = createNativeClient('http://localhost:123/', async () =>
    new Response('internal failure', {status: 500}));
  await assert.rejects(serverError.activate(cap(4), 1, cap(6)), NativeOutcomeUnknown);
  const malformed = createNativeClient('http://localhost:123/', async () =>
    new Response(Uint8Array.of(1, 0, 255), {status: 200}));
  await assert.rejects(malformed.seal(cap(2), cap(3)), NativeOutcomeUnknown);
});
test('authoritative conflict and unauthorized are rejected, ambiguous IO remains UNKNOWN', async () => {
  const rejected = createNativeClient('http://localhost:123/', async () => response(1, Uint8Array.of(10)));
  await assert.rejects(rejected.activate(cap(4), 1n, cap(6)), NativeRejected);
  const unauthorized = createNativeClient('http://localhost:123/', async () => response(1, Uint8Array.of(9)));
  await assert.rejects(unauthorized.register(cap(4), cap(6), Uint8Array.of(1)), NativeRejected);
  const uncertain = createNativeClient('http://localhost:123/', async () => response(1, Uint8Array.of(7)));
  await assert.rejects(uncertain.stage(cap(4), Uint8Array.of(1)), NativeOutcomeUnknown);
});
