import {createServer} from 'node:http';
const backend = process.argv[2].replace(/\/$/, '');
const port = Number(process.argv[3]);
const app = process.argv[4] || 'tools';
createServer(async (req, res) => {
  const address = new URL(req.url, 'http://localhost');
  let path = address.pathname;
  if (path === '/' || path === '/index.html')
    path = '/_skrynia/store/apps/' + app + '/index.html';
  else if (path.startsWith('/releases/') || path === '/active.json')
    path = '/_skrynia/store/apps/' + app + path;
  else if (!path.startsWith('/_skrynia/')) {
    res.writeHead(404).end();
    return;
  }
  try {
    const chunks = [];
    for await (const chunk of req) chunks.push(chunk);
    const headers = {};
    for (const name of ['authorization', 'content-type', 'x-skrynia-if-sha256']) {
      if (req.headers[name]) headers[name] = req.headers[name];
    }
    const response = await fetch(backend + path + address.search, {
      method: req.method, headers, redirect: 'error',
      ...(chunks.length ? {body: Buffer.concat(chunks)} : {}),
    });
    const bytes = Buffer.from(await response.arrayBuffer());
    res.writeHead(response.status, {
      'Content-Type': response.headers.get('content-type') || 'application/octet-stream',
      'Content-Length': bytes.length,
      'Cache-Control': path.includes('/releases/') ? 'public, max-age=31536000, immutable' : 'no-store',
      'Referrer-Policy': 'no-referrer',
    }).end(bytes);
  } catch (error) {
    console.error('fixture error:', error.message);
    res.writeHead(502).end();
  }
}).listen(port, '127.0.0.1');
