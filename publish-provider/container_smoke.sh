#!/usr/bin/env bash
# Real HTTP smoke test against the built container image.
set -euo pipefail

IMAGE="${1:-skrynia-publish-provider:test}"
ENGINE="${CONTAINER_ENGINE:-podman}"
PORT="${SMOKE_PORT:-18787}"
NAME="skrynia-provider-smoke-$$"
ROOT="$(mktemp -d)"
cleanup() {
  "$ENGINE" stop "$NAME" >/dev/null 2>&1 || true
  "$ENGINE" rm "$NAME" >/dev/null 2>&1 || true
  chmod -R u+rwX "$ROOT" >/dev/null 2>&1 || true
  rm -rf "$ROOT" >/dev/null 2>&1 || true
}
trap cleanup EXIT

mkdir -p "$ROOT/requests" "$ROOT/disk/apps/hello" "$ROOT/nginx"
printf 'hello' > "$ROOT/disk/apps/hello/index.html"
printf 'apps/hello/index.html' > "$ROOT/requests/0000002a"
# The image runs as an unprivileged numeric user.
chmod -R a+rwX "$ROOT"

"$ENGINE" run -d --name "$NAME" --network host --read-only \
  -v "$ROOT/requests:/srv/publication-requests:ro" \
  -v "$ROOT/disk:/srv/skrynia-disk:ro" \
  -v "$ROOT/nginx:/srv/nginx-skrynia:rw" \
  -e "PUBLICATION_PROVIDER_BIND=127.0.0.1:$PORT" \
  "$IMAGE" >/dev/null

PORT="$PORT" python3 - <<'PY'
import http.client, json, os, time

port = int(os.environ["PORT"])
def post(path, payload):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
    conn.request("POST", path, body=payload,
                 headers={"Content-Type": "application/octet-stream"})
    response = conn.getresponse()
    body = response.read()
    conn.close()
    assert response.status == 200, (path, response.status)
    return body

def get(path):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
    conn.request("GET", path)
    response = conn.getresponse()
    body = response.read()
    conn.close()
    assert response.status == 200, (path, response.status)
    return body

for _ in range(200):
    try:
        if json.loads(get("/syscalls_list")) == ["publish", "unpublish"]:
            break
    except Exception:
        time.sleep(0.05)
else:
    raise SystemExit("provider did not become ready")

ok = (0).to_bytes(4, "little")
assert post("/publish", (42).to_bytes(4, "little")) == ok
assert post("/publish", (42).to_bytes(4, "little")) == ok
print("container publish ABI OK")
PY

LINK=/srv/nginx-skrynia/apps/hello/index.html
"$ENGINE" exec "$NAME" test -L "$LINK"
"$ENGINE" exec "$NAME" sh -c "test \"\$(readlink '$LINK')\" = '/srv/skrynia-disk/apps/hello/index.html'"
"$ENGINE" exec "$NAME" test -f "$LINK"

PORT="$PORT" python3 - <<'PY'
import http.client, os

port = int(os.environ["PORT"])
def post(path):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
    conn.request("POST", path, body=(42).to_bytes(4, "little"),
                 headers={"Content-Type": "application/octet-stream"})
    response = conn.getresponse()
    body = response.read()
    conn.close()
    assert response.status == 200 and body == (0).to_bytes(4, "little")
post("/unpublish")
post("/unpublish")
print("container unpublish ABI OK")
PY

"$ENGINE" exec "$NAME" test ! -e "$LINK"
"$ENGINE" exec "$NAME" test -f /srv/skrynia-disk/apps/hello/index.html
echo "container filesystem checks OK"
