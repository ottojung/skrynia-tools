export function createBinaryPostTransport(endpoint, fetchImpl = globalThis.fetch) {
  if (typeof endpoint !== 'string' || !/^https?:\/\//.test(endpoint)) {
    throw Error('A valid Program 0 HTTPS endpoint is required');
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
        throw Object.assign(Error('Program 0 request failed; outcome unknown'), {
          code: 'transport_unknown',
        });
      }
      const body = new Uint8Array(await response.arrayBuffer());
      if (response.status < 200 || response.status >= 300) {
        throw Object.assign(Error('Program 0 rejected request (HTTP ' + response.status + ')'), {
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
