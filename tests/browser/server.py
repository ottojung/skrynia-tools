#!/usr/bin/env python3
"""Small isolated Skrynia-like HTTP fixture, not a production server."""
import argparse
import hashlib
import json
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

objects = {}


class Handler(SimpleHTTPRequestHandler):
    def reply(self, code, body=b"", content_type="application/octet-stream"):
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers",
                         "Authorization, X-Skrynia-If-SHA256, Content-Type")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        self.reply(204)

    def do_GET(self):
        route = urlsplit(self.path)
        if route.path == "/health":
            return self.reply(200, b"ok")
        if route.path == "/store":
            matcher = parse_qs(route.query).get("locations", [""])[0].rstrip("*")
            entries = sorted(k for k in objects if k.startswith(matcher))
            return self.reply(200, json.dumps({"keys": entries, "next": None}).encode(),
                              "application/json")
        if route.path.startswith("/store/"):
            key = route.path[len("/store/"):]
            return self.reply(200, objects[key]) if key in objects else self.reply(404)
        return super().do_GET()

    def mutation(self):
        key = urlsplit(self.path).path.removeprefix("/store/")
        if not key or not self.headers.get("Authorization", "").startswith("Bearer "):
            return self.reply(403, b'{"error":"forbidden"}', "application/json")
        size = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(size)
        if self.command == "POST":
            if key in objects:
                return self.reply(409, b'{"error":"already_exists"}', "application/json")
            objects[key] = body
            return self.reply(201)
        if key not in objects:
            return self.reply(404)
        wanted = self.headers.get("X-Skrynia-If-SHA256")
        if wanted is not None and hashlib.sha256(objects[key]).hexdigest() != wanted:
            return self.reply(412, b'{"error":"content_mismatch"}', "application/json")
        if self.command == "DELETE":
            del objects[key]
        else:
            objects[key] = body
        return self.reply(200)

    do_POST = mutation
    do_PUT = mutation
    do_DELETE = mutation


if __name__ == "__main__":
    args = argparse.ArgumentParser()
    args.add_argument("--port", type=int, default=18799)
    options = args.parse_args()
    ThreadingHTTPServer(("127.0.0.1", options.port), Handler).serve_forever()
