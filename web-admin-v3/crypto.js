const encoder = new TextEncoder();
const decoder = new TextDecoder();
const ITERATIONS = 310000;
const VERSION = 1;

function b64url(bytes) {
  let raw = '';
  for (let i = 0; i < bytes.length; i += 8192) {
    raw += String.fromCharCode(...bytes.subarray(i, i + 8192));
  }
  return btoa(raw).replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');
}

function fromB64url(value) {
  const padded = value.replaceAll('-', '+').replaceAll('_', '/')
    .padEnd(Math.ceil(value.length / 4) * 4, '=');
  return Uint8Array.from(atob(padded), c => c.charCodeAt(0));
}

export function randomBytes(count) {
  return crypto.getRandomValues(new Uint8Array(count));
}

export function randomId(bytes = 16) {
  return [...randomBytes(bytes)].map(x => x.toString(16).padStart(2, '0')).join('');
}

export async function digest(bytes) {
  const hash = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  return [...hash].map(x => x.toString(16).padStart(2, '0')).join('');
}

async function deriveKey(password, salt, rounds) {
  const input = await crypto.subtle.importKey(
    'raw', encoder.encode(password), 'PBKDF2', false, ['deriveKey']);
  return crypto.subtle.deriveKey(
    {name: 'PBKDF2', hash: 'SHA-256', salt, iterations: rounds},
    input, {name: 'AES-GCM', length: 256}, false, ['encrypt', 'decrypt']);
}

function aad(context) {
  return encoder.encode('Skrynia Tools v3 admin vault:' + context);
}

export async function encryptVault(value, password, context) {
  const salt = randomBytes(16);
  const iv = randomBytes(12);
  const key = await deriveKey(password, salt, ITERATIONS);
  const data = encoder.encode(JSON.stringify(value));
  const cipher = await crypto.subtle.encrypt(
    {name: 'AES-GCM', iv, additionalData: aad(context)}, key, data);
  return encoder.encode(JSON.stringify({
    schema: VERSION,
    kdf: 'PBKDF2-SHA256',
    rounds: ITERATIONS,
    salt: b64url(salt),
    cipher: 'AES-256-GCM',
    iv: b64url(iv),
    payload: b64url(new Uint8Array(cipher)),
  }));
}

export async function decryptVault(bytes, password, context) {
  if (!(bytes instanceof Uint8Array)) bytes = new Uint8Array(bytes);
  if (bytes.byteLength > 1024 * 1024) throw Error('Vault exceeds 1 MiB');
  let doc;
  try {
    doc = JSON.parse(decoder.decode(bytes));
  } catch {
    throw Error('Vault is not valid JSON');
  }
  if (doc.schema !== VERSION || doc.kdf !== 'PBKDF2-SHA256'
    || doc.cipher !== 'AES-256-GCM' || doc.rounds !== ITERATIONS) {
    throw Error('Unsupported vault format');
  }
  const salt = fromB64url(doc.salt);
  const iv = fromB64url(doc.iv);
  const payload = fromB64url(doc.payload);
  if (salt.length !== 16 || iv.length !== 12) throw Error('Invalid vault envelope');
  if (payload.length > 1024 * 1024) throw Error('Invalid vault envelope');
  const key = await deriveKey(password, salt, doc.rounds);
  let plain;
  try {
    plain = await crypto.subtle.decrypt(
      {name: 'AES-GCM', iv, additionalData: aad(context)}, key, payload);
  } catch {
    throw Error('Unlock failed: invalid access secret or vault data');
  }
  let value;
  try {
    value = JSON.parse(decoder.decode(plain));
  } catch {
    throw Error('Vault contents are not valid JSON');
  }
  if (!value || value.schema !== VERSION) throw Error('Invalid vault contents');
  return value;
}

export const utf8 = text => encoder.encode(text);
export const fromUtf8 = bytes => decoder.decode(bytes);
