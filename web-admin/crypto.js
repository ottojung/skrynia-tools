const encoder = new TextEncoder();
const decoder = new TextDecoder();
const ITERATIONS = 310000;
function binary(value) {
  return Uint8Array.from(atob(value.replaceAll('-', '+').replaceAll('_', '/')
    .padEnd(Math.ceil(value.length / 4) * 4, '=')), c => c.charCodeAt(0));
}
function base64(bytes) {
  let raw = '';
  for (let start = 0; start < bytes.length; start += 8192) {
    raw += String.fromCharCode(...bytes.subarray(start, start + 8192));
  }
  return btoa(raw).replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');
}
export function randomBytes(count) {
  return crypto.getRandomValues(new Uint8Array(count));
}
export function randomId(count = 16) {
  return [...randomBytes(count)].map(x => x.toString(16).padStart(2, '0')).join('');
}
export async function digest(bytes) {
  const hash = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  return [...hash].map(x => x.toString(16).padStart(2, '0')).join('');
}
async function key(password, salt, rounds) {
  const input = await crypto.subtle.importKey('raw', encoder.encode(password),
    'PBKDF2', false, ['deriveKey']);
  return crypto.subtle.deriveKey({name: 'PBKDF2', hash: 'SHA-256',
    salt, iterations: rounds}, input, {name: 'AES-GCM', length: 256},
  false, ['encrypt', 'decrypt']);
}
function aad(vaultKey) {
  return encoder.encode('Skrynia Tools admin vault v1:' + vaultKey);
}
export async function encrypt(value, password, vaultKey) {
  const salt = randomBytes(16);
  const iv = randomBytes(12);
  const secret = await key(password, salt, ITERATIONS);
  const data = encoder.encode(JSON.stringify(value));
  const cipher = await crypto.subtle.encrypt({name: 'AES-GCM', iv,
    additionalData: aad(vaultKey)}, secret, data);
  return encoder.encode(JSON.stringify({schema: 1, kdf: 'PBKDF2-SHA256',
    rounds: ITERATIONS, salt: base64(salt), cipher: 'AES-256-GCM',
    iv: base64(iv), payload: base64(new Uint8Array(cipher))}));
}
export async function decrypt(bytes, password, vaultKey) {
  if (bytes.byteLength > 1024 * 1024) throw Error('Vault exceeds 1 MiB');
  const doc = JSON.parse(decoder.decode(bytes));
  if (doc.schema !== 1 || doc.kdf !== 'PBKDF2-SHA256'
    || doc.cipher !== 'AES-256-GCM' || doc.rounds !== ITERATIONS)
    throw Error('Unsupported vault format');
  const salt = binary(doc.salt);
  const iv = binary(doc.iv);
  const payload = binary(doc.payload);
  if (salt.length !== 16 || iv.length !== 12 || payload.length > 1024 * 1024)
    throw Error('Invalid vault envelope');
  const secret = await key(password, salt, doc.rounds);
  const plain = await crypto.subtle.decrypt({name: 'AES-GCM', iv,
    additionalData: aad(vaultKey)}, secret, payload);
  const value = JSON.parse(decoder.decode(plain));
  if (value.schema !== 1 || typeof value.parentToken !== 'string'
    || !Array.isArray(value.grants)) throw Error('Invalid vault contents');
  return value;
}
export const toBytes = text => encoder.encode(text);
