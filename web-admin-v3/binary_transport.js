// The endpoint is the trusted same-origin gateway, not Program 0 directly.
// The browser posts opaque bytes; the gateway constructs the pinned Program 0
// token frame server-side. No root/issuer/write token belongs in public assets,
// URLs or logs, and this adapter defines no native opcodes.
export function createBinaryPostTransport(endpoint, fetchImpl = globalThis.fetch) {
  if (typeof endpoint !== 'string' || !/^https?:\/\//.test(endpoint)) {
    throw Error('A valid trusted gateway HTTPS endpoint is required');
  }
  if (typeof fetchImpl !== 'function') {
    throw Error('Fetch capability is required');
  }
  const url = endpoint;
  return {
    endpoint: url,
    async post(bytes) {
      if (!(bytes instanceof Uint8Array)) bytes = new Uint8Array(bytes);
      let response;
      try {
        response = await fetchImpl(url, {
          method: 'POST',
          cache: 'no-store',
          redirect: 'error',
          headers: {'Content-Type': 'application/octet-stream'},
          body: bytes,
        });
      } catch {
        throw Object.assign(Error('Gateway request failed; outcome unknown'), {
          code: 'transport_unknown',
        });
      }
      const body = new Uint8Array(await response.arrayBuffer());
      if (response.status < 200 || response.status >= 300) {
        throw Object.assign(Error('Gateway rejected request (HTTP ' + response.status + ')'), {
          code: 'rejected',
          status: response.status,
          body,
        });
      }
      return body;
    },
  };
}

export function createFailClosedNativeProtocol(reason = 'Native Program 0 protocol not frozen') {
  return {
    kind: 'native-protocol-unavailable',
    async request() {
      throw Object.assign(Error(reason), {code: 'protocol_not_frozen'});
    },
  };
}
