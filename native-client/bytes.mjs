export const MAX_BODY = 16 * 1024 * 1024;
export const VERSION = 1;
export function bytes(input, name = 'data') {
  if (!(input instanceof Uint8Array)) throw new TypeError(name + ' must be bytes');
  return input;
}
export function fixed(input, length, name) {
  const value = bytes(input, name);
  if (value.byteLength !== length) throw new RangeError(name + ' must be ' + length + ' bytes');
  return value;
}
export function concat(...parts) {
  const size = parts.reduce((n, part) => n + bytes(part).byteLength, 0);
  if (!Number.isSafeInteger(size) || size > MAX_BODY) throw new RangeError('native POST exceeds 16 MiB');
  const out = new Uint8Array(size);
  let offset = 0;
  for (const part of parts) { out.set(part, offset); offset += part.byteLength; }
  return out;
}
export function u16(n) {
  if (!Number.isInteger(n) || n < 0 || n > 65535) throw new RangeError('u16 out of range');
  return Uint8Array.of(n & 255, n >>> 8);
}
export function u32(n) {
  if (!Number.isInteger(n) || n < 0 || n > 0xffffffff) throw new RangeError('u32 out of range');
  const out = new Uint8Array(4); new DataView(out.buffer).setUint32(0, n, true); return out;
}
export function u64(n) {
  if (typeof n === 'number' && !Number.isSafeInteger(n)) throw new RangeError('unsafe u64 number');
  const v = BigInt(n);
  if (v < 0n || v > 0xffffffffffffffffn) throw new RangeError('u64 out of range');
  const out = new Uint8Array(8); new DataView(out.buffer).setBigUint64(0, v, true); return out;
}
export function id(value) {
  const v = fixed(value, 32, 'capability/id');
  return concat(u16(v.length), v);
}
export function blob(value) {
  const v = bytes(value);
  return concat(u32(v.length), v);
}
export function precond(expected) {
  return expected === null ? Uint8Array.of(0) : concat(Uint8Array.of(1), fixed(expected, 32, 'expected hash'));
}
export function page(after = null, limit = 256) {
  if (!Number.isInteger(limit) || limit < 1 || limit > 256) throw new RangeError('page size 1..256');
  return concat(after === null ? Uint8Array.of(0) :
    concat(Uint8Array.of(1), fixed(after, 32, 'page cursor')), u16(limit));
}
export function packet(opcode, ...fields) {
  if (!Number.isInteger(opcode) || opcode < 0 || opcode > 255) throw new RangeError('invalid opcode');
  return concat(Uint8Array.of(VERSION, opcode), ...fields);
}
export function hex(input) {
  return [...bytes(input)].map(x => x.toString(16).padStart(2, '0')).join('');
}
export function unhex(text, expected = 32) {
  if (typeof text !== 'string' || text.length !== 2 * expected || !/^[0-9a-f]+$/i.test(text))
    throw new TypeError('invalid hexadecimal capability/hash');
  return Uint8Array.from(text.match(/../g), h => parseInt(h, 16));
}
