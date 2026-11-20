(function () {
  async function call(op, payload) {
    const res = await fetch('/_skrynia/', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({op, ...payload}),
    });
    let data;
    try { data = await res.json(); } catch {
      throw Object.assign(Error('Invalid gateway response'), {code: 'invalid_response'});
    }
    if (!res.ok || !data || data.ok !== true) {
      const err = (data && data.error) || {};
      const e = Object.assign(Error(err.message || ('Request failed (' + res.status + ')')), {
        code: err.code || 'error',
      });
      if (err.current !== undefined) e.current = err.current;
      if (err.expected !== undefined) e.expected = err.expected;
      if (err.currentVersion !== undefined) e.currentVersion = err.currentVersion;
      if (err.expectedVersion !== undefined) e.expectedVersion = err.expectedVersion;
      throw e;
    }
    return data.result;
  }

  window.__skryniaV3AdminCapabilities = {
    kind: 'gateway',
    async listApps() { return call('listApps', {}); },
    async getAppState(app) { return call('getAppState', {app}); },
    async getReleaseManifest(app, releaseId) { return call('getReleaseManifest', {app, releaseId}); },
    async activateRelease(app, releaseId, opts) { return call('activateRelease', {app, releaseId, opts: opts || {}}); },
    setOperatorSecret(_secret) { /* no root token in browser; gateway session handled server-side if needed */ },
  };
})();
