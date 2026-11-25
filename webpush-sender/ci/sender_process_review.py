#!/usr/bin/env python3
"""Independent actual-process runtime qualification. No external requests.

Run: python3 sender_process_review.py <path-to-webpush-sender-binary>
All P1 traffic is synthetic loopback; no production config or network is used.
"""
import argparse
import http.server
import json
import os
import pathlib
import struct
import subprocess
import tempfile
import threading
import time

class State:
    calls = 0
    unexpected = False
    lock = threading.Lock()

class Edge(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_POST(self):
        length = int(self.headers.get('Content-Length', '0'))
        body = self.rfile.read(min(length, 4096))
        with State.lock:
            State.calls += 1
            n = State.calls
        if self.path != '/_skrynia/' or body[:2] != bytes([1,20]) or length > 4096:
            State.unexpected = True
            self.send_error(400)
            return
        # First native response is ordinary mutation-lock contention.
        payload = bytes([11]) if n == 1 else bytes(4)
        frame = bytes([1, int(n == 1)]) + struct.pack('<I', len(payload)) + payload
        self.send_response(200)
        self.send_header('Content-Length', str(len(frame)))
        self.send_header('Content-Type','application/octet-stream')
        self.end_headers()
        self.wfile.write(frame)

def config(port, ledger):
    return {
        'edge': {'ip':'127.0.0.1','port':port,'host':'fixture.example.invalid',
                 'origin':'https://fixture.example.invalid','capability':'fixture-sender-edge-only-12345',
                 'timeout_ms':1000,'max_request_bytes':4096,'max_response_bytes':4096},
        'sender_cap':'11'*32,'ns':'22'*32,'key':'wake-fixture','max_payload':4096,
        'max_attempts':2,'base_backoff_ms':10,'max_backoff_ms':1000,'ttl_secs':60,
        'topic':'33'*16,'vapid_sub':'mailto:fixture@example.invalid','ledger_epoch':1,
        'vapid_private_key':'44'*32,'ledger_dir':str(ledger),'poll_ms':250
    }

def write_cfg(path, obj):
    path.write_text(json.dumps(obj))
    path.chmod(0o600)

def wait_calls(p, target, log):
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        if p.poll() is not None:
            raise AssertionError(f'sender exited early ({p.returncode}); {log.read_text()[-300:]}')
        if State.calls >= target:
            return
        time.sleep(.025)
    raise AssertionError('sender did not poll expected number of times')

def no_leak(log):
    text = log.read_text()
    for secret in ['fixture-sender-edge-only-12345', '11'*32, '44'*32]:
        assert secret not in text, 'secret leaked into output'

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('binary', type=pathlib.Path)
    args = ap.parse_args()
    binary = args.binary.resolve()
    assert binary.is_file()
    os.umask(0o077)
    httpd = http.server.ThreadingHTTPServer(('127.0.0.1',0), Edge)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    procs = []
    try:
        with tempfile.TemporaryDirectory(prefix='sender-runtime-review-') as work:
            root = pathlib.Path(work)
            ledger = root/'ledger'
            ledger.mkdir(mode=0o700)
            cfg = root/'config.json'
            obj = config(httpd.server_port, ledger)
            write_cfg(cfg,obj)
            log = root/'sender.log'

            def start(path, logpath):
                with logpath.open('wb') as f:
                    p = subprocess.Popen([str(binary)], env={**os.environ,'WEBPUSH_SENDER_CONFIG':str(path)},
                                         stdout=f,stderr=subprocess.STDOUT)
                procs.append(p)
                return p

            def reject(path, expected, desc):
                before = State.calls
                out = root/'rejected.log'
                p = start(path,out)
                try:
                    rc = p.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    p.kill();p.wait();raise AssertionError(desc+' did not reject within 4 seconds')
                assert rc == expected, (desc, rc, out.read_text()[-150:])
                no_leak(out)
                assert State.calls == before, desc+' unexpectedly sent a request'
                print('PASS',desc,flush=True)

            p = start(cfg,log)
            wait_calls(p,2,log)
            print('PASS transient native Busy=11 then successful empty hint poll',flush=True)
            lock = ledger/'sender.lock'
            inode = lock.stat().st_ino
            # Existing sender still polling; do not assert global call count
            # remains unchanged during its operation.
            otherlog = root/'contender.log'
            contender = start(cfg,otherlog)
            try:
                assert contender.wait(timeout=4) != 0
            except subprocess.TimeoutExpired:
                contender.kill();contender.wait();raise AssertionError('competing process acquired lock')
            no_leak(otherlog)
            assert lock.stat().st_ino == inode
            print('PASS independent process lock contention',flush=True)
            p.kill();p.wait()
            no_leak(log)
            baseline = State.calls
            p2 = start(cfg,root/'restart.log')
            wait_calls(p2,baseline+1,root/'restart.log')
            assert lock.stat().st_ino == inode
            p2.kill();p2.wait()
            no_leak(root/'restart.log')
            print('PASS SIGKILL releases persistent lock without replacing inode',flush=True)
            assert not State.unexpected

            big = root/'big'
            with big.open('wb') as f:
                f.truncate(1<<30)  # sparse, does not allocate a GiB of disk
            reject(big,2,'oversize config rejected before allocation')
            fifo = root/'fifo'
            os.mkfifo(fifo,0o600)
            reject(fifo,2,'config FIFO refused without blocking')
            symlink = root/'symlink'
            symlink.symlink_to(cfg)
            reject(symlink,2,'config symlink refused')
            hardlink = root/'hardlink'
            os.link(cfg,hardlink)
            reject(cfg,2,'config hardlink refused')
            hardlink.unlink()
            cfg.chmod(0o644)
            reject(cfg,2,'nonprivate config refused')
            cfg.chmod(0o600)

            ldlink=root/'ldlink';ldlink.symlink_to(ledger,target_is_directory=True)
            write_cfg(cfg,config(httpd.server_port,ldlink))
            reject(cfg,1,'symlink ledger directory refused')
            for kind in ['symlink','fifo','hardlink','nonprivate']:
                newld=root/('ledger-'+kind);newld.mkdir(mode=0o700)
                path=newld/'sender.lock'
                target=root/('target-'+kind)
                target.write_bytes(b'preserve')
                target.chmod(0o644)
                if kind=='symlink': path.symlink_to(target)
                elif kind=='fifo': os.mkfifo(path,0o600)
                elif kind=='hardlink': os.link(target,path)
                else: path.touch(); path.chmod(0o644)  # umask would otherwise mask mode
                write_cfg(cfg,config(httpd.server_port,newld))
                reject(cfg,1,'unsafe lock '+kind+' refused')
                assert target.read_bytes()==b'preserve' and target.stat().st_mode&0o777==0o644
            print('SENDER_RUNTIME_PROCESS_ALL_OK',flush=True)
    finally:
        for p in procs:
            if p.poll() is None: p.kill();p.wait()
        httpd.shutdown();httpd.server_close();thread.join(timeout=3)

if __name__ == '__main__':
    main()
