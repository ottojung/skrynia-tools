import {request} from './operations.mjs';
import {active, decode, listing, limitedResponse} from './wire.mjs';
import {NativeRejected, NativeOutcomeUnknown, NativeTransportError} from './errors.mjs';
export function secureURL(endpoint) {
  const base = typeof location === 'undefined' ? undefined : location.href;
  const url = new URL(endpoint, base);
  if (url.username || url.password || url.search || url.hash) throw Error('native endpoint must not carry credentials or query');
  const loopback = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname);
  if (url.protocol !== 'https:' && !(url.protocol === 'http:' && loopback))
    throw Error('native endpoint must use HTTPS outside loopback');
  if (typeof location !== 'undefined' && url.origin !== location.origin)
    throw Error('browser native API must use same-origin reverse proxy');
  return url.href;
}
export function createNativeClient(endpoint, fetchFn = fetch) {
  const url = secureURL(endpoint);
  async function call(frame, {mutation = false, signal} = {}) {
    let res; let data;
    try {
      res = await fetchFn(url, {method: 'POST', headers: {'content-type': 'application/octet-stream'},
        body: frame, signal, redirect: 'error', cache: 'no-store',
        credentials: 'omit', referrerPolicy: 'no-referrer'});
      if (res.status !== 200) throw Error('HTTP ' + res.status);
      data = decode(await limitedResponse(res));
    } catch (error) {
      if (mutation) throw new NativeOutcomeUnknown('transport, HTTP, or invalid response', error);
      throw new NativeTransportError('transport, HTTP, or invalid response', error);
    }
    if (data.status === 1) {
      if (data.body.length === 0) {
        if (mutation) throw new NativeOutcomeUnknown('malformed empty error');
        throw new NativeTransportError('malformed empty error');
      }
      const code = data.body[0];
      if (mutation && ![1, 8, 9, 10, 11, 13].includes(code))
        throw new NativeOutcomeUnknown('ambiguous application error ' + code);
      throw new NativeRejected(code, data.body);
    }
    return data.body;
  }
  return Object.freeze({
    call,
    async register(op, admin, wasm, opts) {
      return call(request.register(op, admin, wasm), {...opts, mutation: true});
    },
    async setCode(admin, expectHash, wasm, opts) {
      return call(request.setCode(admin, expectHash, wasm), {...opts, mutation: true});
    },
    async revoke(admin, expectHash, opts) {
      return call(request.revoke(admin, expectHash), {...opts, mutation: true});
    },
    async stage(admin, wasm, opts) {
      return call(request.stageCode(admin, wasm), {...opts, mutation: true});
    },
    async seal(admin, hash, opts) {
      return call(request.sealCode(admin, hash), {...opts, mutation: true});
    },
    async activate(admin, revision, hash, opts) {
      return active(await call(request.activateCode(admin, revision, hash), {...opts, mutation: true}));
    },
    async current(id, opts) { return active(await call(request.getActive(id), opts)); },
    async versions(id, after = null, limit = 256, opts) {
      return listing(await call(request.listVersions(id, after, limit), opts));
    },
    async programs(after = null, limit = 256, opts) {
      return listing(await call(request.listPrograms(after, limit), opts));
    },
    async appPut(cap, key, expect, value, opts) {
      return call(request.appPut(cap, key, expect, value), {...opts, mutation: true});
    },
    async appGet(cap, key, opts) { return call(request.appGet(cap, key), opts); },
    async appDelete(cap, key, hash, opts) {
      return call(request.appDelete(cap, key, hash), {...opts, mutation: true});
    },
    async invoke(id, bytes, opts) { return call(request.invoke(id, bytes), opts); },
    async lookup(id, opts) { return call(request.lookup(id), opts); },
  });
}
