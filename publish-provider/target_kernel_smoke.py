"""Disposable target-kernel/provider integration; no production paths or secrets.

Run with KERNEL_BIN and PROVIDER_BIN pointing to independently built binaries.
The deliberately scalar-only root fixture tests transport, not publication-root.
"""
import hashlib, json, os, pathlib, socket, subprocess, tempfile, time, urllib.request
from contextlib import ExitStack

def uleb(n):
    b=bytearray()
    while n>=128:
        b.append((n&127)|128); n>>=7
    b.append(n); return bytes(b)
def vec(items): return uleb(len(items))+b''.join(items)
def name(s):
    b=s.encode(); return uleb(len(b))+b
def sec(i,b): return bytes([i])+uleb(len(b))+b
def ftype(a,r): return b'\x60'+vec([bytes([x]) for x in a])+vec([bytes([x]) for x in r])
I=0x7f
types=sec(1,vec([ftype([I],[I]),ftype([I,I],[I])]))
imports=sec(2,vec([name(m)+name(n)+b'\0'+uleb(t) for m,n,t in [('skrynia','read1',0),('skrynia','write1',1),('syscall','publish',0),('syscall','unpublish',0)]]))
functions=sec(3,vec([uleb(1)]))
exports=sec(7,vec([name('skrynia_main')+b'\0'+uleb(4)]))
# select publish on byte P (80, signed LEB c0=0xd0 0x00); else unpublish.
body=b'\x00\x20\x01\x20\x00\x10\x00\x41\xd0\x00\x46\x04\x7f\x41\x2a\x10\x02\x05\x41\x2a\x10\x03\x0b\x10\x01\x1a\x41\x00\x0b'
wasm=b'\0asm\x01\0\0\0'+types+imports+functions+exports+sec(10,vec([uleb(len(body))+body]))

def port():
    with socket.socket() as s: s.bind(('127.0.0.1',0)); return s.getsockname()[1]
def http(base,path,data=None):
    r=urllib.request.Request(base+path,data=data,method='POST' if data is not None else 'GET')
    with urllib.request.urlopen(r,timeout=5) as f: return f.read()
def wait(base,path,process,data=None):
    for _ in range(100):
        if process.poll() is not None: raise RuntimeError(f'process exited {process.returncode}')
        try: return http(base,path,data)
        except OSError: time.sleep(.05)
    raise RuntimeError('not ready')
def terminate(p):
    if p.poll() is None:
        p.terminate()
        try:p.wait(timeout=3)
        except subprocess.TimeoutExpired:p.kill();p.wait()
def binary_sha(p):return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
kernel=os.environ['KERNEL_BIN']; provider=os.environ['PROVIDER_BIN']
print('kernel sha256',binary_sha(kernel),'provider sha256',binary_sha(provider),flush=True)
with tempfile.TemporaryDirectory(prefix='skrynia-target-e2e-') as root, ExitStack() as stack:
    r=pathlib.Path(root)
    for p in ['operator','disk','nginx','requests']: (r/p).mkdir()
    (r/'root.wasm').write_bytes(wasm)
    rel='apps/hello/index.html'
    src=r/'disk'/rel; src.parent.mkdir(parents=True);src.write_bytes(b'hello target ABI')
    (r/'requests'/'0000002a').write_bytes(rel.encode())
    pp,kp=port(),port()
    while kp == pp: kp = port()
    pb,kb=f'http://127.0.0.1:{pp}',f'http://127.0.0.1:{kp}'
    penv=dict(os.environ,PUBLICATION_PROVIDER_BIND=f'127.0.0.1:{pp}',PATH_TO_PUBLICATION_REQUESTS=str(r/'requests'),PATH_TO_SKRYNIA_DISK=str(r/'disk'),PATH_TO_NGINX_SKRYNIA_ROOT=str(r/'nginx'))
    kenv=dict(os.environ,SKRYNIA_BIND=f'127.0.0.1:{kp}',SKRYNIA_ROOT_WASM=str(r/'root.wasm'),SKRYNIA_OPERATOR_DIR=str(r/'operator'),SKRYNIA_USER_DIR=str(r/'disk'),SKRYNIA_SYSCALL_URL=pb)
    pl=stack.enter_context(open(r/'provider.log','w'));kl=stack.enter_context(open(r/'kernel.log','w'))
    p=subprocess.Popen([provider],env=penv,stdout=pl,stderr=pl);stack.callback(terminate,p)
    discovery=json.loads(wait(pb,'/syscalls_list',p));print('provider discovery',discovery,flush=True)
    k=subprocess.Popen([kernel],env=kenv,stdout=kl,stderr=kl);stack.callback(terminate,k)
    v=json.loads(wait(kb,'/version',k,b''));print('kernel version',v,flush=True)
    assert v['syscalls']==discovery['functions'],v
    for cmd in [b'P',b'P',b'U',b'U']:
        result=http(kb,'/',cmd);assert result==b'\0',result
        dst=r/'nginx'/rel
        if cmd==b'P':assert dst.is_symlink() and dst.read_bytes()==src.read_bytes()
        else:assert not dst.exists() and not dst.is_symlink()
    assert src.read_bytes()==b'hello target ABI'
    print('PASS: scalar-only WASM root -> actual kernel -> typed HTTP provider -> publish/unpublish effects and idempotence',flush=True)
