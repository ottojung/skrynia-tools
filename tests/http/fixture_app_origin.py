#!/usr/bin/env python3
"""Disposable read-only app origin backed by an actual Skrynia test server."""
import sys
import urllib.error
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BASE = sys.argv[1].rstrip("/")
PORT = int(sys.argv[2])


class Proxy(BaseHTTPRequestHandler):
    def do_GET(self):
        path = urllib.parse.urlsplit(self.path)
        if path.path in ("/", "/index.html"):
            key = "index.html"
        elif path.path in ("/active.json", "/sw.js"):
            key = path.path.lstrip("/")
        elif path.path.startswith("/releases/"):
            key = path.path.lstrip("/")
        elif "text/html" in self.headers.get("Accept", ""):
            key = "index.html"
        else:
            self.send_error(404)
            return
        url = BASE + "/store/apps/demo/" + key
        if path.query:
            url += "?" + path.query
        try:
            with urllib.request.urlopen(url, timeout=10) as reply:
                body = reply.read()
                self.send_response(reply.status)
                self.send_header("Content-Type", reply.headers["Content-Type"])
                self.send_header("Content-Length", str(len(body)))
                self.send_header("Cache-Control", "no-store" if key in
                    ("index.html", "active.json", "sw.js") else
                    "public, max-age=31536000, immutable")
                self.end_headers()
                self.wfile.write(body)
        except urllib.error.HTTPError as error:
            self.send_error(error.code)

    def log_message(self, *_args):
        pass


ThreadingHTTPServer(("127.0.0.1", PORT), Proxy).serve_forever()
