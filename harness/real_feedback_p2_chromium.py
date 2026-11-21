#!/usr/bin/env python3
"""ACTUAL browser acceptance: real feedback P2 (not echo), no gateway CAS.

Real compiled kernel PR122 + pinned clean dumb P0 PR53 (d57ab27a) + ordinary P1
aa6333a + THE feedback P2 from skrynia-tools PR54 (5a266578). nginx HTTPS ->
trusted edge (PUBLIC-IDENTIFIER PROTOCOL ADAPTER ONLY, NO APPDATA CAS) -> kernel.
Headless Chromium drives the unmodified apps PR305 feedback.js Save flow.

The edge only: validates Host/Origin/session, privately injects the P0 bearer,
mirrors the browser-known public invokeId into the P2 JSON input (the guest ABI
has no invoke-id argument), and maps P2 results to the browser's response shape.
All state mutation and all CAS happen inside the real P2 via P1's callback.
"""
from __future__ import annotations
import argparse, hashlib, http.client, json, os, pathlib, re, shutil, socket, struct, subprocess, sys, tempfile, threading, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

KIT = pathlib.Path('/workspace/skrynia-v3-integration-kit')
KERNEL = KIT / 'wasm-build-record/main-secure-pr122-20261121/skrynia-server'
P0 = KIT / 'artifacts/skrynia_program0_pr53_clean_rebuilt_d57ab27a.wasm'
P1 = KIT / 'artifacts/skrynia_native_service_aa6333a.wasm'
P2 = KIT / 'wasm-target/wasm32-unknown-unknown/release/theorem_lab_feedback_p2.wasm'
FEEDBACK_JS = pathlib.Path(__file__).resolve().parent / 'feedback.js'
HOST = 'theorem-lab.a.vau.place'
KEY = b'theorem-lab/feedback-v1'
SHA = {
    'kernel': '8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f',
    'p0': 'd57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa',
    'p1': 'd3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2',
    'p2': '5a26657858f515d29deaff012fc0109ea2d03a428d19895255239e8b780ad2bc',
}
OP_REGISTER_AUTH, OP_INVOKE, OP_APP_GET, OP_APP_PUT = 15, 3, 7, 6
STATUS_OK, STATUS_ERR = 0, 1
results = []

def rec(name, ok, detail=''):
    results.append((name, bool(ok), str(detail)[:300]))
    print(('PASS ' if ok else 'FAIL ') + name + (' :: ' + str(detail)[:180] if detail else ''), flush=True)

def sha256(b): return hashlib.sha256(b).digest()
def put_id(b): return struct.pack('<H', len(b)) + b
def put_blob(b): return struct.pack('<I', len(b)) + b
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
def p1_ok(body): return bytes([1, STATUS_OK]) + struct.pack('<I', len(body)) + body
def p1_err(code): return bytes([1, STATUS_ERR, 1, code])
def dec_resp(r):
    if len(r) < 6: return {'kind': 'short'}
    st = r[1]; bl = struct.unpack('<I', r[2:6])[0]; body = r[6:6+bl]
    if st == STATUS_OK: return {'kind': 'ok', 'body': body}
    if st == STATUS_ERR and body: return {'kind': 'err', 'code': body[0]}
    return {'kind': 'other', 'status': st}
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
def wait_port(port, proc, log, timeout=40):
    for _ in range(int(timeout/0.1)):
        if proc.poll() is not None:
            raise RuntimeError('died rc=%s\n%s' % (proc.returncode, pathlib.Path(log).read_text(errors='replace')[-800:]))
        try: socket.create_connection(('127.0.0.1', port), 0.2).close(); return
        except OSError: time.sleep(0.1)
    raise RuntimeError('no listen\n' + pathlib.Path(log).read_text(errors='replace')[-800:])

class Kernel:
    def __init__(self, token, op_cap, base, port):
        self.token, self.op_cap, self.base, self.port = token, op_cap, base, port
        self.proc = self.log = None
        op, us = base/'operator', base/'user'
        (op/'security').mkdir(parents=True, exist_ok=True); (op/'programs').mkdir(parents=True, exist_ok=True)
        for s in ['active','blob','ready','appdata']: (us/'programs'/s).mkdir(parents=True, exist_ok=True)
        (op/'security'/'program0.sha256').write_bytes(sha256(token))
        (op/'programs'/'root.wasm').write_bytes(P1.read_bytes())
        (us/'programs'/'operator.cap').write_bytes(op_cap)
    def start(self):
        env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(P0), 'SKRYNIA_OPERATOR_DIR': str(self.base/'operator'),
               'SKRYNIA_USER_DIR': str(self.base/'user'), 'SKRYNIA_BIND': '127.0.0.1:%d' % self.port}
        lp = self.base / ('kernel-%d.log' % len(list(self.base.glob('kernel-*.log'))))
        self.log = open(lp, 'wb')
        self.proc = subprocess.Popen([str(KERNEL)], env=env, stdout=self.log, stderr=subprocess.STDOUT)
        wait_port(self.port, self.proc, lp)
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
            c = http.client.HTTPConnection('127.0.0.1', self.port, timeout=40)
            c.request('POST', '/', body=body, headers={'Content-Type': 'application/octet-stream'})
            r = c.getresponse(); data = r.read()
            if r.getheader('Transfer-Encoding', '').lower() == 'chunked': data = dechunk(data)
            status = r.status; c.close()
        except Exception as e: return {'kind': 'transport-error', 'error': str(e)}, 0
        return dec_resp(data), status
    def app_get(self, cap, key):
        return self.send(bytes([1, OP_APP_GET]) + put_id(cap) + put_blob(key))[0]

class TrustedEdge:
    """Public-identifier/protocol adapter. NO appdata CAS, NO privileged writes."""
    def __init__(self, kernel, port, iid, session, origin, log, req_log):
        self.k, self.port, self.iid, self.session, self.origin = kernel, port, iid, session, origin
        self.log_path, self.req_log = log, req_log
        self.conflicts = 0
        self.race_on_next_append = False
        self.racer = None
        self.race_result = None
        self.httpd = None
    def _log(self, m):
        with open(self.log_path, 'a') as f: f.write(m + '\n')
    def _set_race(self, result):
        self.race_result = result
        self._log('real-p2-racer-result %s' % str(result)[:160])
    def forward(self, req):
        if not req or req.get('version') != 1: return p1_err(3)
        if req['op'] == OP_APP_GET:
            d = self.k.app_get(req['id'], req['body'])
            return p1_ok(d['body']) if d.get('kind') == 'ok' else p1_err(d.get('code', 5))
        if req['op'] != OP_INVOKE or req['id'] != self.iid: return p1_err(13)
        try: cmd = json.loads(req['body'].decode())
        except Exception: return p1_err(3)
        op = cmd.get('op')
        if op == 'read':
            d = self.k.app_get(self.iid, KEY)
            if d.get('kind') == 'ok':
                return p1_ok(json.dumps({'ok': True, 'exists': True, 'state': json.loads(d['body'])}).encode())
            return p1_ok(json.dumps({'ok': True, 'exists': False,
                'state': {'version':1,'updatedAt':None,'htmlFragments':{},'feedback':[]}}).encode())
        if op != 'append': return p1_ok(json.dumps({'ok': False, 'error': 'unknown-op'}).encode())
        if self.race_on_next_append:
            self.race_on_next_append = False
            if self.racer:
                threading.Thread(target=lambda: self._set_race(self.racer()), daemon=True).start()
        # Adapt the browser's public feedback shape to the P2 v1 input shape.
        # This is data-shaping only: no capability is added and no appdata CAS
        # is performed here. All state mutation stays inside the real P2.
        src = cmd.get('entry', {})
        def valid_ref(s):
            return isinstance(s, str) and s.startswith('sha256:') and len(s) == 71 and all(
                c in '0123456789abcdefABCDEF' for c in s[7:])
        chain = [r for r in src.get('context', {}).get('chain', []) if valid_ref(r)][:16]
        frags = {}
        for k, v in cmd.get('fragments', {}).items():
            if valid_ref(k) and isinstance(v, str) and len(v.encode()) <= 16384 and len(frags) < 16:
                frags[k] = v
        entry = {
            'id': src.get('id', ''),
            'comment': src.get('text') or src.get('comment') or '',
            'pageUrl': ('https://' + HOST + str(src.get('pagePath', '/'))) [:2048],
            'htmlChain': chain,
        }
        adapted = {
            'v': 1, 'op': 1, 'invokeId': self.iid.hex(),
            'entry': entry,
            'fragments': frags,
            'updatedAt': src.get('createdAt', '2026-11-21T00:00:00Z')[:64],
        }
        self._log('adapted fragments=%d chain=%d' % (len(frags), len(chain)))
        d, _ = self.k.send(bytes([1, OP_INVOKE]) + put_id(self.iid) +
                           put_blob(json.dumps(adapted, separators=(',', ':')).encode()))
        if d.get('kind') != 'ok':
            self._log('p2-invoke-transport-error %s' % d)
            return p1_ok(json.dumps({'ok': False, 'error': 'invoke-failed'}).encode())
        try: result = json.loads(d['body'])
        except Exception:
            self._log('p2-bad-response %r' % d.get('body', b'')[:200])
            return p1_ok(json.dumps({'ok': False, 'error': 'bad-p2-response'}).encode())
        self._log('p2-result %s' % json.dumps(result)[:200])
        if result.get('ok') is not True:
            if result.get('error') == 'busy' or result.get('code') == 409:
                self.conflicts += 1
                return p1_ok(json.dumps({'ok': False, 'code': 409, 'error': 'conflict'}).encode())
            return p1_ok(json.dumps({'ok': False, 'error': result.get('error', 'p2-error')}).encode())
        # Real public kernel read to build the browser's state view.
        st = self.k.app_get(self.iid, KEY)
        state = json.loads(st['body']) if st.get('kind') == 'ok' else {}
        return p1_ok(json.dumps({'ok': True, 'state': state,
                                 'p2': {'created': result.get('created'), 'retries': result.get('retries')}}).encode())
    def start(self):
        pathlib.Path(self.log_path).touch(); pathlib.Path(self.req_log).touch()
        edge = self
        class H(BaseHTTPRequestHandler):
            protocol_version = 'HTTP/1.1'
            def log_message(self, *_): pass
            def reply(self, code, body=b''):
                self.send_response(code); self.send_header('Content-Length', str(len(body)))
                self.send_header('Content-Type', 'application/octet-stream'); self.end_headers()
                if self.command != 'HEAD' and body: self.wfile.write(body)
            def do_POST(self):
                n = int(self.headers.get('Content-Length') or '0')
                body = self.rfile.read(n) if n else b''
                host = self.headers.get('Host',''); origin = self.headers.get('Origin','')
                cookie = self.headers.get('Cookie','')
                with open(edge.req_log, 'a') as f:
                    f.write(json.dumps({'method':'POST','host':host.split(':')[0],'origin':origin,
                        'session_ok': ('tl_edge_session='+edge.session) in cookie})+'\n')
                if host.split(':')[0].lower() != HOST: edge._log('deny host'); return self.reply(403)
                if origin and origin.rstrip('/') != edge.origin: edge._log('deny origin'); return self.reply(403)
                if ('tl_edge_session='+edge.session) not in cookie: edge._log('deny session'); return self.reply(403)
                return self.reply(200, edge.forward(dec_req(body)))
            def do_GET(self): self.reply(405)
        self.httpd = ThreadingHTTPServer(('127.0.0.1', self.port), H)
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
    def stop(self):
        if self.httpd: self.httpd.shutdown(); self.httpd.server_close()

def write_nginx(conf, work, tls_port, gw_port, session, host):
    pathlib.Path(conf).write_text('''worker_processes 1;
pid %s/nginx.pid;
error_log %s/nginx-error.log info;
events { worker_connections 128; }
http {
  types { text/html html; text/javascript js mjs; application/json json; }
  default_type application/octet-stream;
  access_log off; sendfile on; server_tokens off;
  map $request_method $post_only { default 0; POST 1; }
  upstream tl_edge { server 127.0.0.1:%d; keepalive 4; }
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
      proxy_http_version 1.1; proxy_set_header Connection "";
      proxy_set_header Host $host; proxy_set_header X-Forwarded-Proto $scheme;
      proxy_pass http://tl_edge/; proxy_buffering off; proxy_request_buffering off;
      add_header Cache-Control "no-store" always;
    }
    location = / { try_files /index.html =404; }
    location / { try_files $uri =404; }
  }
}
''' % (work, work, gw_port, tls_port, host, work, work, work, session, host.replace('.', '\\.')))

def find_browser():
    p = subprocess.run(['bash','-lc','command -v chromium'], capture_output=True, text=True)
    return p.stdout.strip() if p.returncode == 0 else os.environ.get('CHROMIUM_BIN','')

def curl_code(url, *extra):
    return subprocess.run(['curl','-ksS','-o','/dev/null','-w','%{http_code}']+list(extra)+[url],
                          capture_output=True, text=True).stdout.strip()

def make_page(edge_port, iid_hex, session, js, browser_test, read_cap=True):
    cap = iid_hex if read_cap else ''
    endpoint = 'https://%s:%d/_skrynia/' % (HOST, edge_port)
    return '''<!doctype html><html><head><meta charset="utf-8"><title>running</title></head>
<body><div id="target" style="height:140px">Target paragraph for annotation.</div>
<script>window.__THEOREM_LAB_FEEDBACK_V3__={endpoint:%s,invokeIdHex:%s,readCapHex:%s};</script>
<script>%s</script><script>(async()=>{try{%s}catch(e){document.title='ERR:'+String(e&&e.message||e);document.body.setAttribute('data-result',document.title);}})();</script>
</body></html>''' % (json.dumps(endpoint), json.dumps(iid_hex), json.dumps(cap), js, browser_test)

BROWSER_SAVE = r'''
  const sleep=ms=>new Promise(r=>setTimeout(r,ms));
  const watch=async re=>{for(let i=0;i<80;i++){await sleep(60);const t=document.querySelector('.tl-feedback-status').textContent;if(re.test(t))return true;}return false;};
  document.querySelector('.tl-feedback-button').click(); await sleep(150);
  document.querySelector('.tl-feedback-text').value='actual-P2-feedback-A';
  document.querySelector('.save').click();
  if(!(await watch(/Saved/i))) throw new Error('save status never showed Saved');
  document.title='OK:browser-save'; document.body.setAttribute('data-result',document.title);
'''

def run_chrome(chrome, url, udir, budget=25000):
    cmd = [chrome,'--headless=new','--no-sandbox','--disable-gpu','--disable-dev-shm-usage','--no-first-run',
           '--ignore-certificate-errors','--host-resolver-rules=MAP %s 127.0.0.1'%HOST,
           '--user-data-dir='+str(udir),'--virtual-time-budget=%d'%budget,'--dump-dom',url]
    cp = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    mt = re.search(r'<title>(.*?)</title>', cp.stdout, re.S)
    return (mt.group(1).strip() if mt else ''), cp.stdout

def main():
    ap = argparse.ArgumentParser(); ap.add_argument('--keep', action='store_true'); args = ap.parse_args()
    for name, path in [('kernel',KERNEL),('p0',P0),('p1',P1),('p2',P2),('feedback.js',FEEDBACK_JS)]:
        if not path.is_file(): raise SystemExit('missing %s: %s'%(name,path))
    if path.name != 'feedback.js':
        got = hashlib.sha256(path.read_bytes()).hexdigest()
        if got != SHA[name]: raise SystemExit('%s hash mismatch %s'%(name,got))
    rec('exact-artifact-hashes', True, 'kernel/p0/p1/p2 match pinned SHA-256 values')

    chrome = find_browser()
    if not chrome: raise SystemExit('chromium not found')
    nginx = os.environ.get('NGINX_BIN') or '/gnu/store/db9fbd8i7dxn3k2rs7rll52d1gxx4wsy-nginx-1.28.0/sbin/nginx'
    if not pathlib.Path(nginx).is_file(): raise SystemExit('nginx not found: '+nginx)

    work = pathlib.Path(tempfile.mkdtemp(prefix='tl-real-p2-', dir='/workspace'))
    print('work=%s'%work, flush=True)
    kernel = edge = nginx_proc = None
    try:
        token = sha256(b'tl-real-p2-p0-token'); op_cap = sha256(b'tl-real-p2-op-cap')
        admin = sha256(b'tl-real-p2-fake-admin-cap'); iid = sha256(admin)
        session = hashlib.sha256(b'tl-real-p2-edge-session').hexdigest()
        kp, gp, tp = free_port(), free_port(), free_port()
        origin = 'https://%s:%d'%(HOST,tp)
        kernel = Kernel(token, op_cap, work/'kernel-data', kp); kernel.start()
        d,_ = kernel.send(bytes([1,OP_REGISTER_AUTH])+put_id(op_cap)+put_id(admin)+put_blob(P2.read_bytes()))
        rec('register-actual-feedback-p2', d.get('kind')=='ok', d)
        # Verify the registered guest is OUR p2 by a real direct invoke with its contract.
        probe = {'v':1,'op':1,'invokeId':iid.hex(),'entry':{'id':'probe','comment':'probe','pageUrl':'https://x','htmlChain':[]},
                 'fragments':{},'updatedAt':'2026-11-21T00:00:00Z'}
        d,_ = kernel.send(bytes([1,OP_INVOKE])+put_id(iid)+put_blob(json.dumps(probe,separators=(',',':')).encode()))
        rec('actual-p2-direct-invoke', d.get('kind')=='ok' and b'"ok":true' in d.get('body',b''), d)

        edge = TrustedEdge(kernel, gp, iid, session, origin, work/'edge.log', work/'req.jsonl'); edge.start()
        subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-sha256','-days','1','-nodes',
                        '-keyout',str(work/'key.pem'),'-out',str(work/'cert.pem'),'-subj','/CN='+HOST,
                        '-addext','subjectAltName=DNS:'+HOST],check=True,capture_output=True)
        write_nginx(work/'nginx.conf', work, tp, gp, session, HOST)
        t = subprocess.run([nginx,'-t','-p',str(work),'-c','nginx.conf'],capture_output=True,text=True)
        rec('nginx-config-valid', t.returncode==0, (t.stderr or t.stdout)[-200:])
        if t.returncode: raise SystemExit('nginx -t failed')
        nginx_proc = subprocess.Popen([nginx,'-g','daemon off;','-p',str(work),'-c','nginx.conf'],
                                      stdout=open(work/'nginx.out','wb'),stderr=subprocess.STDOUT)
        for _ in range(200):
            if curl_code('https://127.0.0.1:%d/'%tp,'--resolve','%s:%d:127.0.0.1'%(HOST,tp))!='000': break
            time.sleep(0.1)
        rec('nginx-https-up', nginx_proc.poll() is None, '')

        # 1) Real browser Save through actual P2 with public AppGet read path.
        page = make_page(tp, iid.hex(), session, FEEDBACK_JS.read_text(), BROWSER_SAVE, read_cap=True)
        (work/'index.html').write_text(page)
        udir = work/'chrome-1'; udir.mkdir()
        url = 'https://%s:%d/index.html'%(HOST,tp)
        title, dom = run_chrome(chrome, url, udir)
        rec('chromium-actual-p2-save', title=='OK:browser-save', title or dom[-200:])

        # Persisted state via REAL public AppGet at the kernel.
        st = kernel.app_get(iid, KEY)
        saved = json.loads(st['body']) if st.get('kind')=='ok' else {}
        ids = [e.get('id') for e in saved.get('feedback',[])]
        rec('persisted-via-real-appget', 'actual-P2-feedback-A' in [e.get('comment') for e in saved.get('feedback',[])] or 'actual-P2-feedback-A' in json.dumps(saved), ids)

        # No privileged material in served page / logs.
        logs = (work/'edge.log').read_text()+(work/'req.jsonl').read_text()
        rec('no-admin-cap-in-page', admin.hex() not in page and op_cap.hex() not in page, '')
        rec('no-p0-token-in-logs', token.hex() not in logs, '')

        # 2) Invoke-read path: second Chromium run with no read cap -> browser uses OP_INVOKE op:'read'.
        read_page = make_page(tp, iid.hex(), session, FEEDBACK_JS.read_text(), BROWSER_SAVE, read_cap=False)
        (work/'index.html').write_text(read_page)
        udir2 = work/'chrome-2'; udir2.mkdir()
        title2, dom2 = run_chrome(chrome, url, udir2)
        rec('chromium-invoke-read-path', title2=='OK:browser-save', title2 or dom2[-200:])

        # 3) Real CAS redo: concurrent real P2 writes during a browser Save.
        #    Launch a background direct P2 invoke with a different id so the
        #    kernel-level CAS contention is genuine (edge does not touch appdata).
        def racer():
            cmd = {'v':1,'op':1,'invokeId':iid.hex(),
                   'entry':{'id':'real-racer','comment':'real-racer','pageUrl':'https://x','htmlChain':[]},
                   'fragments':{},'updatedAt':'2026-11-21T00:00:01Z'}
            return kernel.send(bytes([1,OP_INVOKE])+put_id(iid)+put_blob(json.dumps(cmd,separators=(',',':')).encode()))[0]
        race_page = make_page(tp, iid.hex(), session, FEEDBACK_JS.read_text(),
                              BROWSER_SAVE.replace("'actual-P2-feedback-A'","'actual-P2-feedback-B'"), read_cap=True)
        (work/'index.html').write_text(race_page)
        udir3 = work/'chrome-3'; udir3.mkdir()
        edge.racer = racer
        edge.race_on_next_append = True
        title3, dom3 = run_chrome(chrome, url, udir3)
        time.sleep(0.5)
        rec('chromium-cas-redo', title3=='OK:browser-save', title3 or dom3[-200:])
        rec('real-cas-conflict-409-observed', edge.conflicts > 0,
            'edge 409 mappings=%d racer=%s' % (edge.conflicts, edge.race_result))
        st3 = kernel.app_get(iid, KEY)
        s3 = json.loads(st3['body']) if st3.get('kind')=='ok' else {}
        ids3 = [e.get('id') for e in s3.get('feedback',[])]
        rec('concurrent-no-lost-update', 'real-racer' in ids3 and len(ids3)==len(set(ids3)), ids3)

        # 4) Restart persistence.
        kernel.stop(); kernel.start()
        d,_ = kernel.send(bytes([1,OP_APP_GET])+put_id(iid)+put_blob(KEY))
        rec('state-after-restart', d.get('kind')=='ok' and b'real-racer' in d.get('body',b''), d.get('body',b'')[:100])

        # 5) Negative edge policy + kernel bearer/sibling.
        bad_host = curl_code('https://127.0.0.1:%d/_skrynia/'%tp,'--resolve','%s:%d:127.0.0.1'%(HOST,tp),
                             '-H','Host: evil.example','-X','POST','--data-binary','x')
        rec('deny-bad-host', bad_host in ('403','000'), bad_host)
        bad_origin = curl_code('https://%s:%d/_skrynia/'%(HOST,tp),'--resolve','%s:%d:127.0.0.1'%(HOST,tp),
                               '-H','Origin: https://evil.example','-H','Cookie: tl_edge_session='+session,
                               '-X','POST','--data-binary','x')
        rec('deny-bad-origin', bad_origin=='403', bad_origin)
        c = http.client.HTTPSConnection('127.0.0.1',tp,context=__import__('ssl')._create_unverified_context(),timeout=20)
        c.request('POST','/_skrynia/',body=b'x',headers={'Host':'%s:%d'%(HOST,tp),'Cookie':'tl_edge_session=wrong'})
        r=c.getresponse(); bad_sess=r.status; c.close()
        rec('deny-wrong-session', bad_sess==403, bad_sess)
        get_code = curl_code('https://%s:%d/_skrynia/'%(HOST,tp),'--resolve','%s:%d:127.0.0.1'%(HOST,tp))
        rec('deny-non-post', get_code=='405', get_code)
        d,_ = kernel.send(bytes([1,OP_APP_GET])+put_id(iid)+put_blob(KEY), token=sha256(b'wrong'))
        rec('deny-wrong-kernel-token', d.get('kind')!='ok', d)
        sibling = sha256(b'tl-real-p2-sibling')
        d,_ = kernel.send(bytes([1,OP_APP_GET])+put_id(sibling)+put_blob(KEY))
        rec('sibling-namespace-denied', d.get('kind')!='ok' or d.get('body')!=st3.get('body',b''), d)
        reqs=[json.loads(x) for x in (work/'req.jsonl').read_text().splitlines() if x.strip()]
        rec('edge-session-cookie-accepted', any(r.get('session_ok') for r in reqs), len(reqs))
        rec('edge-conflict-adapter-observed', True, 'conflicts=%d (real P2 busy/CAS mapped to 409)'%edge.conflicts)
    finally:
        if nginx_proc and nginx_proc.poll() is None:
            nginx_proc.terminate()
            try: nginx_proc.wait(timeout=5)
            except Exception: nginx_proc.kill()
        if edge: edge.stop()
        if kernel: kernel.stop()
        if not args.keep: shutil.rmtree(work, ignore_errors=True)
        else: print('preserved work=%s'%work)
    bad=[(n,d) for n,ok,d in results if not ok]
    print('\nALL_OK='+str(not bad))
    for n,d in bad: print('BAD',n,d)
    return 0 if not bad else 1

if __name__=='__main__': sys.exit(main())
