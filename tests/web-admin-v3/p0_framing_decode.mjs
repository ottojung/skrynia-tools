// Fail-closed decode test for the pinned token-gating Program 0 binary frame.
//
// Reference artifact: skrynia-tools Program 0 WASM
//   8a03330da0c9b1235f5866a41f1e2e9d1ef1732770753387521b5db904f7d217
// built from agent/persist-svc-callback-fixes-qW2Emr2jXayvHtelVsFoFP-voronezh
// @ ea4f152. Mirrors program0/src/proxy/auth/mod.rs::require and
// program0/src/token.rs::matches.
//
// This is a wire-contract test, not backend evidence: the verifier is an
// injected server-only capability, no secret is embedded in this file, and the
// injected fake capability adapter is deliberately not used here. It does not
// prove that a live Program 0, gateway or ordinary service exists.
import assert from 'node:assert/strict';
import {createHash, timingSafeEqual} from 'node:crypto';

const VERSION = 1;
const TOKEN_BYTES = 32;
const HEADER_BYTES = 1 + TOKEN_BYTES;

function sha256(bytes) {
  return new Uint8Array(createHash('sha256').update(bytes).digest());
}

function frameError(code, message) {
  return Object.assign(Error(message), {code});
}

// Mirrors auth::require: read exactly one version byte + 32 raw token bytes,
// reject any other version, then constant-time SHA256(raw) == private verifier.
// The remainder is returned opaque; no root opcode is decoded or invented.
async function decodeP0RequestFrame(body, verifierCapability) {
  if (!(body instanceof Uint8Array)) throw frameError('invalid_frame', 'Body must be bytes');
  if (body.length < HEADER_BYTES) throw frameError('invalid_frame', 'Truncated P0 header');
  if (body[0] !== VERSION) throw frameError('invalid_version', 'Unsupported P0 frame version');
  const verifier = await verifierCapability();
  if (!(verifier instanceof Uint8Array) || verifier.length !== TOKEN_BYTES) {
    throw frameError('invalid_verifier', 'Invalid P0 token verifier');
  }
  const digest = sha256(body.subarray(1, HEADER_BYTES));
  if (!timingSafeEqual(Buffer.from(digest), Buffer.from(verifier))) {
    throw frameError('denied', 'P0 bearer token denied');
  }
  return body.slice(HEADER_BYTES);
}

function frame(version, token, payload = new Uint8Array()) {
  const body = new Uint8Array(HEADER_BYTES + payload.length);
  body[0] = version;
  body.set(token, 1);
  body.set(payload, HEADER_BYTES);
  return body;
}

const token = new Uint8Array(TOKEN_BYTES);
for (let i = 0; i < TOKEN_BYTES; i += 1) token[i] = (i * 7 + 3) & 0xff;
const verifier = sha256(token);
const verifierCapability = async () => verifier;

// 1. A valid frame yields the opaque remainder byte-for-byte. Bytes that were
// legacy v2 "opcodes" (0x07/0x08) stay opaque: P0 decodes no root opcodes.
const payload = new Uint8Array([0x07, 0x08, 0x01, 0x00, 0xff, 0x01]);
const decoded = await decodeP0RequestFrame(frame(VERSION, token, payload), verifierCapability);
assert.deepEqual([...decoded], [...payload]);
assert.notDeepEqual([...decoded.subarray(0, TOKEN_BYTES)], [...token]);

// 2. A header-only frame is valid and decodes to an empty opaque payload.
assert.equal((await decodeP0RequestFrame(frame(VERSION, token), verifierCapability)).length, 0);

async function expectFail(name, body, verifierFn, expectedCode) {
  await assert.rejects(
    () => decodeP0RequestFrame(body, verifierFn),
    err => {
      assert.equal(err.code, expectedCode, `${name}: wrong code (${err.code})`);
      // No token or verifier material may leak into an error/log string.
      const text = JSON.stringify({message: err.message, code: err.code});
      assert.ok(!text.includes(Buffer.from(token).toString('hex')), `${name}: token leaked`);
      assert.ok(!text.includes(Buffer.from(verifier).toString('hex')), `${name}: verifier leaked`);
      return true;
    },
    name,
  );
}

// 3. Only version 0x01 is accepted.
for (const bad of [0, 2, 0xff]) {
  await expectFail(`version ${bad} denied`, frame(bad, token), verifierCapability, 'invalid_version');
}

// 4. Truncated/short bodies fail closed before any token comparison.
for (const n of [0, 1, 31, 32]) {
  await expectFail(`truncated body ${n}`, new Uint8Array(n), verifierCapability, 'invalid_frame');
}

// 5. A wrong token, a single-bit tamper, and the verifier-as-token mistake
// (authenticate the raw secret, not the digest) are all denied.
const wrong = Uint8Array.from(token);
wrong[0] ^= 1;
await expectFail('wrong token denied', frame(VERSION, wrong), verifierCapability, 'denied');
await expectFail('verifier as token denied', frame(VERSION, verifier), verifierCapability, 'denied');
await expectFail('zero token denied', frame(VERSION, new Uint8Array(TOKEN_BYTES)), verifierCapability, 'denied');

// 6. A missing, short or extra-byte private verifier fails closed (read::verifier
// requires exactly 32 bytes and no trailing byte).
await expectFail('short verifier denied', frame(VERSION, token), async () => verifier.subarray(0, 31), 'invalid_verifier');
await expectFail('long verifier denied', frame(VERSION, token), async () => Uint8Array.from([...verifier, 0]), 'invalid_verifier');

console.log('v3 pinned-P0 binary framing fail-closed decode tests passed');
