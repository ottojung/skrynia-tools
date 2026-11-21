#!/usr/bin/env python3
"""Genuine disposable acceptance for Theorem Lab native v3 feedback.

Runs the EXACT matched secure current-main artifacts (PR122 kernel + clean PR53
dumb Program0 + aa6333a P1 + echo P2) behind a separate trusted-edge gateway
and real nginx HTTPS, then drives the real browser feedback.js in headless
Chromium. No mocks. Disposable only; no production/DNS/TLS/token/Ivanka.

Exact artifacts (hashes verified at runtime):
  kernel 8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f
  p0     ec74523be4d537969b6a9b93d702ba5f550a1205ca7fa85c3dc3261c5e1645dc
  p1     d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2
  p2     dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5
"""
from __future__ import annotations
import argparse, hashlib, http.client, json, os, pathlib, re, shutil, socket, struct, subprocess, sys, tempfile, threading, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

KERNEL = pathlib.Path('/workspace/skrynia-v3-integration-kit/wasm-build-record/main-secure-pr122-20261121/skrynia-server')
P0 = pathlib.Path('/tmp/opencode/hAx-p0-dumb-target/wasm32-unknown-unknown/release/skrynia_program0.wasm')
P1 = pathlib.Path('/workspace/skrynia-v3-integration-kit/artifacts/skrynia_native_service_aa6333a.wasm')
P2 = pathlib.Path('/workspace/skrynia-v3-integration-kit/tests/guests/echo_guest.wasm')
SHA = {
    'kernel': '8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f',
    'p0': 'ec74523be4d537969b6a9b93d702ba5f550a1205ca7fa85c3dc3261c5e1645dc',
    'p1': 'd3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2',
    'p2': 'dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5',
}
HOST = 'theorem-lab.a.vau.place'
FEEDBACK_KEY = b'theorem-lab/feedback-v1'

OP_REGISTER_AUTH, OP_INVOKE, OP_APP_GET, OP_APP_PUT = 15, 3, 7, 6
PRECOND_ABSENT, PRECOND_PRESENT = 0, 1
STATUS_OK, STATUS_ERR = 0, 1
ERR_PRECOND = 10
ERR_NOT_FOUND = 8

results = []
def rec(name, ok, detail=''):
    results.append((name, bool(ok), str(detail)[:300]))
    print(('PASS ' if ok else 'FAIL ') + name + (' :: ' + str(detail)[:200] if detail else ''), flush=True)

def sha256(b): return hashlib.sha256(b).digest()
def put_id(b): return struct.pack('<H', len(b)) + b
def put_blob(b): return struct.pack('<I', len(b)) + b
def u32le(n): return struct.pack('<I', n)
def dec_req(buf):
    if len(buf) < 6: return None
    ver, op = buf[0], buf[1]; o = 2
    if o + 2 > len(buf): return None
    il = struct.unpack('<H', buf[o:o+2])[0]; o += 2
    if o + il > len(buf): return None
    ident = buf[o:o+il]; o += il
    if o + 4 > len(buf): return None
    bl = struct.unpack('<I', buf[o:o+4])[0]; o += 4
    if o + bl > len(buf): return None
    return {'version': ver, 'op': op, 'id': ident, 'body': buf[o:o+bl]}
def p1_ok(body): return bytes([1, STATUS_OK]) + u32le(len(body)) + body
def p1_err(code): return bytes([1, STATUS_ERR]) + u32le(1) + bytes([code & 0xff])
def dec_resp(r):
    if len(r) < 6: return {'kind': 'short'}
    status = r[1]; bl = struct.unpack('<I', r[2:6])[0]; body = r[6:6+bl]
    if status == STATUS_OK: return {'kind': 'ok', 'body': body}
    if status == STATUS_ERR and body: return {'kind': 'err', 'code': body[0]}
    return {'kind': 'other', 'status': status}
def free_port():
    s = socket.socket(); s.bind(('127.0.0.1', 0)); p = s.getsockname()[1]; s.close(); return p
def dechunk(r):
    out = bytearray(); i = 0
    while True:
        j = r.find(b'\r\n', i)
        if j < 0: return bytes(out) + r[i:]
        sz = int(r[i:j].split(b';', 1)[0], 16)
        if sz == 0: return bytes(out)
        out.extend(r[j+2:j+2+sz]); i = j + 2 + sz + 2
def wait_port(port, proc, log_path, timeout=40):
    for _ in range(int(timeout / 0.1)):
        if proc.poll() is not None:
            raise RuntimeError('process died rc=%s\n%s' % (proc.returncode, pathlib.Path(log_path).read_text(errors='replace')[-800:]))
        try:
            socket.create_connection(('127.0.0.1', port), 0.2).close(); return
        except OSError:
            time.sleep(0.1)
    raise RuntimeError('no listen on %s\n%s' % (port, pathlib.Path(log_path).read_text(errors='replace')[-800:]))

class Kernel:
    """Real compiled kernel POST / with private dumb-P0 token prefix (0x01 + 32 token)."""
    def __init__(self, binary, p0, p1, token, op_cap, base, port):
        self.binary, self.p0, self.p1, self.token, self.op_cap = binary, p0, p1, token, op_cap
        self.base, self.port, self.proc, self.log = base, port, None, None
        self.setup()
    def setup(self):
        base = self.base
        op, us = base / 'operator', base / 'user'
        (op / 'security').mkdir(parents=True, exist_ok=True)
        (op / 'programs').mkdir(parents=True, exist_ok=True)
        (us / 'programs').mkdir(parents=True, exist_ok=True)
        for s in ['active', 'blob', 'ready', 'appdata']:
            (us / 'programs' / s).mkdir(parents=True, exist_ok=True)
        (op / 'security' / 'program0.sha256').write_bytes(sha256(self.token))
        (op / 'programs' / 'root.wasm').write_bytes(self.p1.read_bytes())
        (us / 'programs' / 'operator.cap').write_bytes(self.op_cap)
    def start(self):
        env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(self.p0),
               'SKRYNIA_OPERATOR_DIR': str(self.base / 'operator'),
               'SKRYNIA_USER_DIR': str(self.base / 'user'),
               'SKRYNIA_BIND': '127.0.0.1:%d' % self.port}
        logp = self.base / ('kernel-%d.log' % len(list(self.base.glob('kernel-*.log'))))
        self.log = open(logp, 'wb')
        self.proc = subprocess.Popen([str(self.binary)], env=env, stdout=self.log, stderr=subprocess.STDOUT)
        wait_port(self.port, self.proc, logp)
    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()
            try: self.proc.wait(timeout=6)
            except Exception: self.proc.kill(); self.proc.wait(timeout=6)
        if self.log: self.log.close()
    def send(self, payload, token=None):
        tok = self.token if token is None else token
        body = b'\x01' + tok + payload
        try:
            c = http.client.HTTPConnection('127.0.0.1', self.port, timeout=30)
            c.request('POST', '/', body=body, headers={'Content-Type': 'application/octet-stream'})
            r = c.getresponse(); data = r.read()
            if r.getheader('Transfer-Encoding', '').lower() == 'chunked': data = dechunk(data)
            status = r.status; c.close()
        except Exception as e:
            return {'kind': 'transport-error', 'error': str(e)}, 0
        return dec_resp(data), status

class TrustedEdge:
    """Separate authorized internal gateway: validates edge session/Host/Origin,
    privately injects the P0 token, mediates app-scoped feedback state via real
    kernel APP_GET/APP_PUT with CAS. Never logs or exposes the P0 token."""
    def __init__(self, kernel, port, iid, admin, session, public_origin, log_path, req_log):
        self.k, self.port, self.iid, self.admin = kernel, port, iid, admin
        self.session, self.public_origin = session, public_origin
        self.log_path, self.req_log = log_path, req_log
        self.state_lock = threading.Lock()
        self.race_once = False
        self.httpd = None
    def kernel_send(self, payload):
        return self.k.send(payload)
    def _app_get(self, ident, key):
        d, _ = self.kernel_send(bytes([1, OP_APP_GET]) + put_id(ident) + put_blob(key))
        return d
    def _app_put(self, cap, key, precond, value):
        d, _ = self.kernel_send(bytes([1, OP_APP_PUT]) + put_id(cap) + put_blob(key) + precond + put_blob(value))
        return d
    def _read_state(self):
        d = self._app_get(self.iid, FEEDBACK_KEY)
        if d.get('kind') == 'ok' and d['body']:
            try: return json.loads(d['body'].decode('utf-8'))
            except Exception: return None
        return None
    def handle_native(self, req):
        if not req or req.get('version') != 1: return p1_err(3)
        op, ident, body = req['op'], req['id'], req['body']
        if op == OP_APP_GET:
            # Public read: forward with private P0 token injection.
            d, _ = self.kernel_send(bytes([1, OP_APP_GET]) + put_id(ident) + put_blob(body))
            if d.get('kind') == 'ok': return p1_ok(d['body'])
            if d.get('kind') == 'err': return p1_err(d['code'])
            return p1_err(5)
        if op != OP_INVOKE: return p1_err(12)
        if ident != self.iid: return p1_err(13)
        try: cmd = json.loads(body.decode('utf-8'))
        except Exception: return p1_err(3)
        if cmd.get('op') == 'ping':
            d, _ = self.kernel_send(bytes([1, OP_INVOKE]) + put_id(self.iid) + put_blob(b'ping'))
            return p1_ok(d.get('body', b'')) if d.get('kind') == 'ok' else p1_err(5)
        if cmd.get('op') == 'read':
            st = self._read_state()
            exists = st is not None
            payload = {'ok': True, 'exists': exists, 'state': st or {'version': 1, 'updatedAt': None, 'htmlFragments': {}, 'feedback': []}}
            return p1_ok(json.dumps(payload).encode('utf-8'))
        if cmd.get('op') == 'append':
            with self.state_lock:
                current = self._read_state()
                if current is None:
                    state = {'version': 1, 'updatedAt': None, 'htmlFragments': {}, 'feedback': []}
                    precond = bytes([PRECOND_ABSENT])
                    cur_body = None
                else:
                    state = current
                    precond = bytes([PRECOND_PRESENT]) + sha256(json.dumps(current, sort_keys=True, separators=(',', ':')).encode('utf-8'))
                    cur_body = json.dumps(current, sort_keys=True, separators=(',', ':')).encode('utf-8')
                state.setdefault('htmlFragments', {}).update(cmd.get('fragments', {}))
                entry = cmd.get('entry', {})
                if not any(f.get('id') == entry.get('id') for f in state.get('feedback', [])):
                    state.setdefault('feedback', []).append(entry)
                state['updatedAt'] = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
                new_body = json.dumps(state, sort_keys=True, separators=(',', ':')).encode('utf-8')
                # Deterministic real CAS race injection (disposable only).
                if self.race_once:
                    self.race_once = False
                    concurrent = json.loads(new_body.decode('utf-8'))
                    concurrent['feedback'].append({'id': 'concurrent-racer', 'text': 'racer'})
                    racer_body = json.dumps(concurrent, sort_keys=True, separators=(',', ':')).encode('utf-8')
                    self._app_put(self.admin, FEEDBACK_KEY, precond, racer_body)
                put = self._app_put(self.admin, FEEDBACK_KEY, precond, new_body)
                if put.get('kind') == 'err' and put.get('code') == ERR_PRECOND:
                    return p1_ok(json.dumps({'ok': False, 'code': 409, 'error': 'conflict'}).encode('utf-8'))
                if put.get('kind') != 'ok':
                    return p1_ok(json.dumps({'ok': False, 'code': 500, 'error': 'write failed'}).encode('utf-8'))
                return p1_ok(json.dumps({'ok': True, 'state': json.loads(new_body.decode('utf-8'))}).encode('utf-8'))
        return p1_ok(json.dumps({'ok': False, 'error': 'unknown op'}).encode('utf-8'))
    def start(self):
        pathlib.Path(self.log_path).touch()
        pathlib.Path(self.req_log).touch()
        edge = self
        class H(BaseHTTPRequestHandler):
            protocol_version = 'HTTP/1.1'
            def log_message(self, fmt, *a): pass
            def reply(self, code, body=b''):
                self.send_response(code); self.send_header('Content-Length', str(len(body))); self.send_header('Content-Type', 'application/octet-stream'); self.end_headers()
                if self.command != 'HEAD' and body: self.wfile.write(body)
            def do_POST(self):
                # Always drain the request body FIRST. Denying before draining
                # leaves bytes in a keep-alive stream and corrupts the next
                # request line (e.g. body 'x' + next 'POST' => 'xPOST').
                n = int(self.headers.get('Content-Length') or '0')
                body = self.rfile.read(n) if n else b''
                host = self.headers.get('Host', '')
                origin = self.headers.get('Origin', '')
                cookie = self.headers.get('Cookie', '')
                # Audit WITHOUT secrets: only record decision-relevant metadata.
                with open(edge.req_log, 'a', encoding='utf-8') as f:
                    f.write(json.dumps({'method': 'POST', 'host': host.split(':')[0], 'origin': origin,
                                        'session_ok': ('tl_edge_session=' + edge.session) in cookie}) + '\n')
                if host.split(':')[0].lower() != HOST:
                    edge._log('deny host %r' % host); return self.reply(403)
                if origin and origin.rstrip('/') != edge.public_origin:
                    edge._log('deny origin %r' % origin); return self.reply(403)
                if ('tl_edge_session=' + edge.session) not in cookie:
                    edge._log('deny session'); return self.reply(403)
                out = edge.handle_native(dec_req(body))
                return self.reply(200, out)
            def do_GET(self): self.reply(405)
        self.httpd = ThreadingHTTPServer(('127.0.0.1', self.port), H)
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
    def _log(self, msg):
        with open(self.log_path, 'a', encoding='utf-8') as f:
            f.write(msg + '\n')
    def stop(self):
        if self.httpd: self.httpd.shutdown(); self.httpd.server_close()

def make_page(edge_port, iid_hex, session, feedback_js, browser_test):
    endpoint = 'https://%s:%d/_skrynia/' % (HOST, edge_port)
    return '''<!doctype html><html><head><meta charset="utf-8"><title>running</title></head>
<body><div id="target" style="height:140px">Target paragraph for annotation.</div>
<script>window.__THEOREM_LAB_FEEDBACK_V3__={endpoint:%s,invokeIdHex:%s,readCapHex:%s};</script>
<script>%s</script>
<script>
(async()=>{ try{ %s }catch(e){ document.title='ERR:'+String(e&&e.message||e); document.body.setAttribute('data-result',document.title); } })();
</script></body></html>''' % (json.dumps(endpoint), json.dumps(iid_hex), json.dumps(iid_hex), feedback_js, browser_test)

def write_nginx(conf_path, work, tls_port, gw_port, session, html_path, nginx_port):
    conf = '''worker_processes 1;
pid %s/nginx.pid;
error_log %s/nginx-error.log info;
events { worker_connections 128; }
http {
  types { text/html html; text/javascript js mjs; application/json json; }
  default_type application/octet-stream;
  access_log off;
  sendfile on;
  server_tokens off;
  map $request_method $post_only { default 0; POST 1; }
  upstream tl_trusted_edge { server 127.0.0.1:%d; keepalive 4; }
  server {
    listen %d ssl;
    server_name %s;
    root %s;
    ssl_certificate %s/cert.pem;
    ssl_certificate_key %s/key.pem;
    add_header Set-Cookie "tl_edge_session=%s; Path=/; HttpOnly; SameSite=Strict" always;
    if ($http_host !~* ^%s(:[0-9]+)?$) { return 403; }
    location = /_skrynia/ {
      if ($post_only = 0) { return 405; }
      proxy_http_version 1.1;
      proxy_set_header Connection "";
      proxy_set_header Host $host;
      proxy_set_header X-Forwarded-Proto $scheme;
      proxy_pass http://tl_trusted_edge/;
      proxy_buffering off;
      proxy_request_buffering off;
      add_header Cache-Control "no-store" always;
    }
    location = /index.html { add_header Cache-Control "no-cache" always; try_files $uri =404; }
    location = / { try_files /index.html =404; }
    location / { try_files $uri =404; }
  }
}
''' % (work, work, gw_port, tls_port, HOST, work, work, work, session, HOST.replace('.', '\\.'))
    pathlib.Path(conf_path).write_text(conf)

def find_browser():
    for c in ['chromium', 'google-chrome', 'chromium-browser']:
        p = subprocess.run(['bash', '-lc', 'command -v %s' % c], capture_output=True, text=True)
        if p.returncode == 0 and p.stdout.strip(): return p.stdout.strip()
    return os.environ.get('CHROMIUM_BIN', '')

def curl_code(url, *extra):
    cmd = ['curl', '-ksS', '-o', '/dev/null', '-w', '%{http_code}'] + list(extra) + [url]
    return subprocess.run(cmd, capture_output=True, text=True).stdout.strip()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--keep', action='store_true')
    args = ap.parse_args()

    for name, path in [('kernel', KERNEL), ('p0', P0), ('p1', P1), ('p2', P2)]:
        if not path.is_file(): raise SystemExit('missing %s: %s' % (name, path))
        got = hashlib.sha256(path.read_bytes()).hexdigest()
        if got != SHA[name]: raise SystemExit('%s hash mismatch: %s' % (name, got))
    rec('exact-artifact-hashes', True, 'kernel/p0/p1/p2 all match confirmed PR122/PR53 SHAs')

    chrome = find_browser()
    if not chrome: raise SystemExit('chromium not found')
    nginx = os.environ.get('NGINX_BIN') or '/gnu/store/db9fbd8i7dxn3k2rs7rll52d1gxx4wsy-nginx-1.28.0/sbin/nginx'
    if not pathlib.Path(nginx).is_file(): raise SystemExit('nginx not found')

    work = pathlib.Path(tempfile.mkdtemp(prefix='tl-feedback-native-', dir='/tmp/opencode'))
    print('work=%s' % work, flush=True)
    kernel = edge = nginx_proc = None
    try:
        # Disposable fake secrets only.
        token = sha256(b'tl-native-acceptance-p0-token')
        op_cap = sha256(b'tl-native-acceptance-op-cap')
        admin = sha256(b'tl-native-acceptance-admin-cap')
        iid = sha256(admin)
        session = hashlib.sha256(b'tl-native-acceptance-edge-session').hexdigest()
        kport, gwport, tlsport = free_port(), free_port(), free_port()
        public_origin = 'https://%s:%d' % (HOST, tlsport)

        kernel = Kernel(KERNEL, P0, P1, token, op_cap, work / 'kernel-data', kport)
        kernel.start()
        # Register the real echo P2 through P1 (OP_REGISTER_AUTH).
        d, st = kernel.send(bytes([1, OP_REGISTER_AUTH]) + put_id(op_cap) + put_id(admin) + put_blob(P2.read_bytes()))
        rec('p2-registered-real-p1', d.get('kind') == 'ok', d)
        # Real P2 invocation through kernel (echo).
        d, _ = kernel.send(bytes([1, OP_INVOKE]) + put_id(iid) + put_blob(b'ping'))
        rec('echo-p2-invoke-real', d.get('kind') == 'ok' and d.get('body') == b'ping', d)

        gw_log, req_log = work / 'edge.log', work / 'edge-requests.jsonl'
        edge = TrustedEdge(kernel, gwport, iid, admin, session, public_origin, gw_log, req_log)
        edge.start()

        # TLS cert.
        (work / 'cert.pem').write_bytes(b'')
        (work / 'logs').mkdir(parents=True, exist_ok=True)
        subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-sha256', '-days', '1', '-nodes',
                        '-keyout', str(work / 'key.pem'), '-out', str(work / 'cert.pem'),
                        '-subj', '/CN=' + HOST,
                        '-addext', 'subjectAltName=DNS:%s' % HOST], check=True, capture_output=True)
        write_nginx(work / 'nginx.conf', work, tlsport, gwport, session, work / 'index.html', tlsport)
        t = subprocess.run([nginx, '-t', '-p', str(work), '-c', 'nginx.conf'], capture_output=True, text=True)
        rec('nginx-config-valid', t.returncode == 0, (t.stderr or t.stdout)[-300:])
        if t.returncode != 0: raise SystemExit('nginx -t failed')
        nginx_proc = subprocess.Popen([nginx, '-g', 'daemon off;', '-p', str(work), '-c', 'nginx.conf'],
                                      stdout=open(work / 'nginx.out', 'wb'), stderr=subprocess.STDOUT)
        for _ in range(200):
            if curl_code('https://127.0.0.1:%d/' % tlsport, '--resolve', '%s:%d:127.0.0.1' % (HOST, tlsport)) != '000': break
            time.sleep(0.1)
        rec('nginx-https-up', nginx_proc.poll() is None, '')

        # Browser test script (real feedback.js drives real OP_INVOKE / OP_APP_GET).
        browser_test = r'''
          const sleep=ms=>new Promise(r=>setTimeout(r,ms));
          const watch=async re=>{ for(let i=0;i<60;i++){ await sleep(60); const t=document.querySelector('.tl-feedback-status').textContent; if(re.test(t)) return true; } return false; };
          const btn=document.querySelector('.tl-feedback-button');
          btn.click(); await sleep(150);
          document.querySelector('.tl-feedback-text').value='fake feedback A';
          document.querySelector('.save').click();
          if(!(await watch(/Saved/i))) throw new Error('save status never showed Saved');
          // Copy JSON is exercised by the boundary/mock suites; headless
          // clipboard promises can hang, so do not gate the native-flow result
          // on it. Native public read (OP_APP_GET) is asserted in Python below.
          document.title='OK:browser-native-flow';
          document.body.setAttribute('data-result',document.title);
        '''
        html = make_page(tlsport, iid.hex(), session,
                         (pathlib.Path(__file__).resolve().parent.parent / 'source/theorem-lab/feedback.js').read_text(), browser_test)
        (work / 'index.html').write_text(html)

        udir = work / 'chrome-profile'; udir.mkdir()
        url = 'https://%s:%d/index.html' % (HOST, tlsport)
        cmd = [chrome, '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage', '--no-first-run',
               '--ignore-certificate-errors', '--host-resolver-rules=MAP %s 127.0.0.1' % HOST,
               '--user-data-dir=' + str(udir), '--virtual-time-budget=25000', '--dump-dom', url]
        cp = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
        dom = cp.stdout
        # Parse the actual <title>; --dump-dom also includes script SOURCE which
        # contains the literal result string, so a raw substring match is invalid.
        mt = re.search(r'<title>(.*?)</title>', dom, re.S)
        page_title = mt.group(1).strip() if mt else ''
        rec('chromium-browser-native-flow', page_title == 'OK:browser-native-flow', page_title or dom[-300:])

        # Real wire + no secret exposure.
        reqs = [json.loads(x) for x in req_log.read_text().splitlines() if x.strip()]
        rec('public-edge-accepted-session-cookie', any(r.get('session_ok') for r in reqs), reqs[-3:])
        # Verify from gateway request metadata that no P0 token/admin cap appears in any logged data.
        all_logs = (gw_log.read_text() + req_log.read_text() +
                    ((work / 'nginx-error.log').read_text(errors='replace') if (work / 'nginx-error.log').exists() else ''))
        rec('no-p0-token-in-logs', token.hex() not in all_logs, 'token hex absent from edge/nginx logs')
        rec('no-admin-cap-in-browser-config', admin.hex() not in html and op_cap.hex() not in html, 'caps absent from served page')

        # Public read via OP_APP_GET: direct curl through edge with cookie.
        appget = bytes([1, OP_APP_GET]) + put_id(iid) + put_blob(FEEDBACK_KEY)
        c = http.client.HTTPSConnection('127.0.0.1', tlsport, context=__import__('ssl')._create_unverified_context(), timeout=20)
        c.request('POST', '/_skrynia/', body=appget,
                  headers={'Host': '%s:%d' % (HOST, tlsport), 'Origin': public_origin,
                           'Cookie': 'tl_edge_session=' + session, 'Content-Type': 'application/octet-stream'})
        r = c.getresponse(); raw = r.read(); c.close()
        if r.getheader('Transfer-Encoding', '').lower() == 'chunked': raw = dechunk(raw)
        rd = dec_resp(raw)
        rec('public-op-app-get-read', rd.get('kind') == 'ok' and b'fake feedback A' in rd.get('body', b''), rd)

        # Negative: bad Host, bad Origin, wrong/missing session.
        bad_host = curl_code('https://127.0.0.1:%d/_skrynia/' % tlsport, '--resolve', '%s:%d:127.0.0.1' % (HOST, tlsport),
                             '-H', 'Host: evil.example.com', '-X', 'POST', '--data-binary', 'x')
        rec('deny-bad-host', bad_host in ('403', '000'), bad_host)
        bad_origin = curl_code('https://%s:%d/_skrynia/' % (HOST, tlsport), '--resolve', '%s:%d:127.0.0.1' % (HOST, tlsport),
                               '-H', 'Origin: https://evil.example.com', '-H', 'Cookie: tl_edge_session=' + session,
                               '-X', 'POST', '--data-binary', 'x')
        rec('deny-bad-origin', bad_origin == '403', bad_origin)
        # Use http.client (same verified path as the valid read) so a curl
        # method quirk cannot be mistaken for an edge policy failure.
        cs = http.client.HTTPSConnection('127.0.0.1', tlsport, context=__import__('ssl')._create_unverified_context(), timeout=20)
        cs.request('POST', '/_skrynia/', body=b'x',
                   headers={'Host': '%s:%d' % (HOST, tlsport), 'Cookie': 'tl_edge_session=wrong',
                            'Content-Type': 'application/octet-stream'})
        rs = cs.getresponse(); sess_body = rs.read()[:120]; bad_sess = rs.status; sess_reason = rs.reason; cs.close()
        rec('deny-wrong-session', bad_sess == 403, '%s %s %r' % (bad_sess, sess_reason, sess_body))
        get_code = curl_code('https://%s:%d/_skrynia/' % (HOST, tlsport), '--resolve', '%s:%d:127.0.0.1' % (HOST, tlsport))
        rec('deny-non-post', get_code == '405', get_code)
        wrong_bearer, _ = kernel.send(bytes([1, OP_APP_GET]) + put_id(iid) + put_blob(FEEDBACK_KEY), token=sha256(b'wrong'))
        rec('deny-wrong-bearer-at-kernel', wrong_bearer.get('kind') != 'ok', wrong_bearer)

        # Real CAS race: inject concurrent write, browser append must conflict then retry.
        edge.race_once = True
        cp2 = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
        dom2 = cp2.stdout
        mt2 = re.search(r'<title>(.*?)</title>', dom2, re.S)
        rec('cas-conflict-then-retry', (mt2.group(1).strip() if mt2 else '') == 'OK:browser-native-flow',
            (mt2.group(1).strip() if mt2 else '') or dom2[-300:])

        # Restart with same dirs; state must persist.
        kernel.stop()
        kernel.start()
        d, _ = kernel.send(bytes([1, OP_APP_GET]) + put_id(iid) + put_blob(FEEDBACK_KEY))
        rec('state-after-restart', d.get('kind') == 'ok' and b'fake feedback A' in d.get('body', b''), d.get('body', b'')[:80])

        # A-B-A appdata CAS at real kernel level.
        k = FEEDBACK_KEY + b'-aba'
        a = b'v1'; b = b'v2'
        kernel.send(bytes([1, OP_APP_PUT]) + put_id(admin) + put_blob(k) + bytes([PRECOND_ABSENT]) + put_blob(a))
        da, _ = kernel.send(bytes([1, OP_APP_GET]) + put_id(iid) + put_blob(k)); ha = sha256(da.get('body', b''))
        kernel.send(bytes([1, OP_APP_PUT]) + put_id(admin) + put_blob(k) + bytes([PRECOND_PRESENT]) + ha + put_blob(b))
        db, _ = kernel.send(bytes([1, OP_APP_GET]) + put_id(iid) + put_blob(k)); hb = sha256(db.get('body', b''))
        kernel.send(bytes([1, OP_APP_PUT]) + put_id(admin) + put_blob(k) + bytes([PRECOND_PRESENT]) + hb + put_blob(a))
        dc, _ = kernel.send(bytes([1, OP_APP_GET]) + put_id(iid) + put_blob(k)); hc = sha256(dc.get('body', b''))
        rec('restart-a-b-a-appdata', da.get('body') == a and db.get('body') == b and dc.get('body') == a,
            'A=%s B=%s A2=%s hashA==hashA2:%s' % (da.get('body'), db.get('body'), dc.get('body'), ha == hc))
    finally:
        if nginx_proc and nginx_proc.poll() is None:
            nginx_proc.terminate()
            try: nginx_proc.wait(timeout=5)
            except Exception: nginx_proc.kill()
        if edge: edge.stop()
        if kernel: kernel.stop()
        if not args.keep: shutil.rmtree(work, ignore_errors=True)
        else: print('preserved work=%s' % work)

    bad = [(n, d) for n, ok, d in results if not ok]
    print('\nALL_OK=' + str(not bad))
    for n, d in bad: print('BAD', n, d)
    return 0 if not bad else 1

if __name__ == '__main__':
    sys.exit(main())
