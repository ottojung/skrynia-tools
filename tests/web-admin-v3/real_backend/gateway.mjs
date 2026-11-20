import http from 'node:http';
import {readFile, stat} from 'node:fs/promises';
import {join, extname, normalize} from 'node:path';
import {fileURLToPath} from 'node:url';

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
};

function send(res, status, body, type = 'text/plain; charset=utf-8') {
  const buf = Buffer.isBuffer(body) ? body : Buffer.from(String(body));
  res.writeHead(status, {
    'Content-Type': type,
    'Content-Length': buf.length,
    'Cache-Control': 'no-store',
    'Referrer-Policy': 'no-referrer',
  });
  res.end(buf);
}

function safeJoin(root, urlPath) {
  const clean = normalize(urlPath).replace(/^([/\\])+/, '');
  if (clean.includes('..')) return null;
  return join(root, clean);
}

export function createGatewayServer({service, webRoot, extraRoots = {}, port = 0, host = '127.0.0.1'}) {
  let testResult = null;
  const server = http.createServer(async (req, res) => {
    try {
      const u = new URL(req.url || '/', 'http://localhost');
      if (req.method === 'POST' && u.pathname === '/__test_result') {
        const chunks = [];
        for await (const c of req) chunks.push(c);
        try { testResult = JSON.parse(Buffer.concat(chunks).toString('utf8')); } catch { testResult = {ok:false,detail:'invalid json'}; }
        return send(res, 200, JSON.stringify({ok:true}), 'application/json; charset=utf-8');
      }
      if (req.method === 'GET' && u.pathname === '/__test_result') {
        return send(res, 200, JSON.stringify(testResult), 'application/json; charset=utf-8');
      }
      if (req.method === 'POST' && u.pathname === '/_skrynia/') {
        // Dumb transport: opaque body forwarded to nonzero userland registry service.
        // No application/registry semantics in this gateway path.
        const chunks = [];
        for await (const c of req) chunks.push(c);
        const raw = Buffer.concat(chunks);
        let payload;
        try { payload = JSON.parse(raw.toString('utf8')); }
        catch { return send(res, 400, JSON.stringify({ok:false,error:{message:'Invalid request body',code:'invalid_input'}}), 'application/json; charset=utf-8'); }
        try {
          const result = await service.handleRequest(payload);
          return send(res, 200, JSON.stringify({ok: true, result}), 'application/json; charset=utf-8');
        } catch (e) {
          const status = e && (e.code === 'not_found') ? 404 : e && e.code === 'conflict' ? 409 : 400;
          return send(res, status, JSON.stringify({ok: false, error: {message: e.message || 'Error', code: e.code || 'error', current: e.current, expected: e.expected, currentVersion: e.currentVersion, expectedVersion: e.expectedVersion, currentRevision: e.currentRevision, expectedRevision: e.expectedRevision}}), 'application/json; charset=utf-8');
        }
      }
      if (req.method !== 'GET' && req.method !== 'HEAD') {
        return send(res, 405, 'Method not allowed');
      }
      let path = u.pathname === '/' ? '/index.html' : u.pathname;
      let root = webRoot;
      for (const [prefix, dir] of Object.entries(extraRoots)) {
        if (path === prefix || path.startsWith(prefix + '/')) {
          root = dir;
          path = path.slice(prefix.length) || '/';
          if (path === '/') path = '/index.html';
          break;
        }
      }
      const file = safeJoin(root, path);
      if (!file) return send(res, 403, 'Forbidden');
      let st;
      try { st = await stat(file); } catch { return send(res, 404, 'Not found'); }
      if (!st.isFile()) return send(res, 404, 'Not found');
      const data = await readFile(file);
      const type = MIME[extname(file)] || 'application/octet-stream';
      res.writeHead(200, {'Content-Type': type, 'Content-Length': data.length, 'Cache-Control': 'no-store'});
      if (req.method === 'HEAD') res.end(); else res.end(data);
    } catch (e) {
      try { send(res, 500, 'Internal error'); } catch {}
    }
  });
  return {
    server,
    getTestResult: () => testResult,
    resetTestResult: () => { testResult = null; },
    listen: () => new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(port, host, () => {
        const addr = server.address();
        resolve({port: addr.port, host: addr.address, origin: `http://${addr.address}:${addr.port}`});
      });
    }),
    close: () => new Promise((resolve, reject) => server.close(err => err ? reject(err) : resolve())),
  };
}
