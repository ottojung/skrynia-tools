import {packet, id, blob, fixed, precond, page, u64} from './bytes.mjs';
export const request = Object.freeze({
  register: (operatorCap, adminCap, wasm) => packet(15, id(operatorCap), id(adminCap), blob(wasm)),
  setCode: (adminCap, expect, wasm) => packet(2, id(adminCap), precond(expect), blob(wasm)),
  invoke: (invokeId, input) => packet(3, id(invokeId), blob(input)),
  lookup: invokeId => packet(4, id(invokeId)),
  revoke: (adminCap, expect) => packet(5, id(adminCap), precond(expect)),
  appPut: (writeCap, key, expect, value) =>
    packet(6, id(writeCap), blob(key), precond(expect), blob(value)),
  appGet: (readCap, key) => packet(7, id(readCap), blob(key)),
  appDelete: (writeCap, key, expectedHash) =>
    packet(8, id(writeCap), blob(key), fixed(expectedHash, 32, 'expected hash')),
  stageCode: (adminCap, wasm) => packet(9, id(adminCap), blob(wasm)),
  sealCode: (adminCap, hash) => packet(10, id(adminCap), fixed(hash, 32, 'code hash')),
  activateCode: (adminCap, revision, hash) =>
    packet(11, id(adminCap), u64(revision), fixed(hash, 32, 'ready hash')),
  getActive: invokeId => packet(12, id(invokeId)),
  listVersions: (invokeId, after = null, limit = 256) => packet(13, id(invokeId), page(after, limit)),
  listPrograms: (after = null, limit = 256) => packet(14, page(after, limit)),
});
