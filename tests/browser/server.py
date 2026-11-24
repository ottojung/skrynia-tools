#!/usr/bin/env python3
"""Disposable binary-protocol fixture, not a production service."""
import argparse
import hashlib
import struct
import threading
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

objects = {}
lock = threading.Lock()
WRITE = bytes([42]) * 32
READ = hashlib.sha256(WRITE).digest()

def parse(frame):
    def take(n):
        nonlocal frame
        if len(frame) < n:
            raise ValueError("truncated")
        result, frame = frame[:n], frame[n:]
        return result
    def blob(n):
        return take(int.from_bytes(take(n), "little"))
    version, op = take(2)
    if version != 1 or op not in (6, 7, 8):
        raise ValueError("opcode")
    cap, key = blob(2), blob(4).decode("ascii")
    if cap != (READ if op == 7 else WRITE):
        return 1, b"\x09", key
    if op == 7:
        if frame:
            raise ValueError("trailing")
        return (0, objects[key], key) if key in objects else (1, b"\x08", key)
    expected = take(32) if op == 8 else (take(32) if take(1) == b"\x01" else None)
    value = blob(4) if op == 6 else None
    if frame:
        raise ValueError("trailing")
    old = objects.get(key)
    current = hashlib.sha256(old).digest() if old is not None else None
    if expected != current:
        return 1, b"\x0a\x00", key
    if op == 8:
        if old is None:
            return 1, b"\x08", key
        del objects[key]
        return 0, b"", key
    if key != "test.accepted":
        objects[key] = value
    return 0, hashlib.sha256(value).digest(), key

class Handler(SimpleHTTPRequestHandler):
    def reply(self, code, body=b""):
        self.send_response(code)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            return self.reply(200, b"ok")
        return super().do_GET()

    def do_POST(self):
        if self.path != "/_skrynia/" or self.headers.get("X-Skrynia-Edge-Capability") != "test-edge":
            return self.reply(403)
        try:
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 <= size <= 16 * 1024 * 1024:
                raise ValueError("size")
            frame = self.rfile.read(size)
            with lock:
                status, body, key = parse(frame)
            if key == "test.partial":
                return self.reply(206, b"partial")
            if key == "test.accepted":
                return self.reply(202)
            return self.reply(200, struct.pack("<BBI", 1, status, len(body)) + body)
        except (ValueError, UnicodeError):
            return self.reply(400)

if __name__ == "__main__":
    args = argparse.ArgumentParser()
    args.add_argument("--port", type=int, default=18799)
    ThreadingHTTPServer(("127.0.0.1", args.parse_args().port), Handler).serve_forever()
