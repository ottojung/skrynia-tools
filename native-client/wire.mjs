import {bytes, fixed, MAX_BODY, VERSION} from './bytes.mjs';
export function decode(raw) {
  const v = bytes(raw, 'response');
  if (v.length < 6 || v[0] !== VERSION || (v[1] !== 0 && v[1] !== 1))
    throw Error('invalid native response header');
  const n = new DataView(v.buffer, v.byteOffset + 2, 4).getUint32(0, true);
  if (n !== v.length - 6 || n > MAX_BODY) throw Error('invalid native response length');
  return {status: v[1], body: v.slice(6)};
}
export function active(bytesIn) {
  const v = fixed(bytesIn, 42, 'active pointer');
  if (v[0] !== 3 || v[1] > 1) throw Error('invalid active pointer schema');
  const revision = new DataView(v.buffer, v.byteOffset + 34, 8).getBigUint64(0, true);
  if (revision === 0n) throw Error('invalid zero activation revision');
  return {live: v[1] === 1, hash: v.slice(2, 34), revision};
}
export function listing(bytesIn) {
  const v = bytes(bytesIn);
  if (v.length < 2) throw Error('short list response');
  const count = new DataView(v.buffer, v.byteOffset, 2).getUint16(0, true);
  if (v.length !== 2 + count * 32 || count > 256) throw Error('invalid list response');
  return Array.from({length: count}, (_, i) => v.slice(2 + i * 32, 34 + i * 32));
}
export async function limitedResponse(response, max = MAX_BODY + 6) {
  const stream = response.body;
  if (!stream || typeof stream.getReader !== 'function') {
    const data = new Uint8Array(await response.arrayBuffer());
    if (data.byteLength > max) throw Error('oversized native response');
    return data;
  }
  const reader = stream.getReader();
  const chunks = []; let size = 0;
  try {
    while (true) {
      const {value, done} = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > max) throw Error('oversized native response');
      chunks.push(value);
    }
  } catch (error) {
    await reader.cancel().catch(() => {});
    throw error;
  } finally { reader.releaseLock(); }
  const all = new Uint8Array(size); let p = 0;
  for (const chunk of chunks) { all.set(chunk, p); p += chunk.length; }
  return all;
}
