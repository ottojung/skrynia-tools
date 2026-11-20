export {createNativeClient, secureURL} from './client.mjs';
export {request} from './operations.mjs';
export {decode, active, listing} from './wire.mjs';
export {hex, unhex, fixed, bytes} from './bytes.mjs';
export {NativeRejected, NativeOutcomeUnknown, NativeTransportError} from './errors.mjs';
export async function sha256(data) {
  if (!(data instanceof Uint8Array)) throw new TypeError('digest input must be bytes');
  return new Uint8Array(await crypto.subtle.digest('SHA-256', data));
}
