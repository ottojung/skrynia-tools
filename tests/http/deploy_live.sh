#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
server="$SKRYNIA_TEST_SERVER_BIN"
bin="$repo/target/debug/skrynia-tools"
tmp="$(mktemp -d)"
pid=""
proxy_pid=""
cleanup() {
  if [ -n "$proxy_pid" ]; then kill "$proxy_pid" 2>/dev/null || true; wait "$proxy_pid" 2>/dev/null || true; fi
  if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi
  rm -rf "$tmp"
}
trap cleanup EXIT
mkdir "$tmp/root" "$tmp/site"
port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
url="http://127.0.0.1:$port"
SKRYNIA_URL="$url" SKRYNIA_DATA_DIR="$tmp/root" SKRYNIA_PORT="$port" \
  SKRYNIA_HOST=127.0.0.1 "$server" > "$tmp/server.log" 2>&1 &
pid="$!"
for _ in $(seq 1 80); do
  if curl -fsS "$url/health" > /dev/null 2>&1; then break; fi
  if ! kill -0 "$pid" 2>/dev/null; then cat "$tmp/server.log"; exit 1; fi
  sleep 0.1
done
SKRYNIA_DATA_DIR="$tmp/root" "$server" root-credentials > "$tmp/root.json"
python3 - "$url" "$tmp/root.json" "$tmp/user-token" "$tmp/root-token" <<'PY'
import json,sys,urllib.request
base, credentials, result, root_token=sys.argv[1:]
root=json.load(open(credentials,encoding="utf8"))
def post(path, token, body):
  req=urllib.request.Request(base+path,data=json.dumps(body).encode(),
    headers={"Authorization":"Bearer "+token,"Content-Type":"application/json"},method="POST")
  with urllib.request.urlopen(req,timeout=10) as response:
    return json.load(response)
parent=post("/grants/"+root["id"]+"/users",root["issuerToken"],{})
with open(root_token,"w",encoding="utf8") as stream: stream.write(parent["userToken"])
grant=post("/grants",parent["userToken"],{
  "locations":"apps/demo/*","operations":["create","modify","delete","list"],
  "maxBytes":5000000,"maxObjects":300})
child=post("/grants/"+grant["id"]+"/users",grant["issuerToken"],{})
with open(result,"w",encoding="utf8") as stream: stream.write(child["userToken"])
PY
export SKRYNIA_TOKEN="$(cat "$tmp/user-token")"
mkdir -p "$tmp/site/assets"
cat > "$tmp/site/index.html" <<'HTML'
<!doctype html><html><body>
<h1 id="app">waiting</h1><script src="./assets/app.js"></script>
<script>document.documentElement.dataset.url =
  location.pathname + location.search + location.hash;</script></body></html>
HTML
printf "document.getElementById('app').textContent='HTTP release one';" > "$tmp/site/assets/app.js"
mkdir -p "$tmp/site/docs"
cat > "$tmp/site/docs/about.html" <<'HTML'
<!doctype html><html><body><h1>Nested page works</h1><script>
document.documentElement.dataset.href=location.pathname+location.search+location.hash;
</script></body></html>
HTML

# Simulate the old stable entrypoint: a deploy must upgrade it safely.
curl -fsS -X POST "$url/store/apps/demo/index.html" \
  -H "Authorization: Bearer $SKRYNIA_TOKEN" \
  --data-binary 'outdated stable bootstrap' >/dev/null
sha=1111111111111111111111111111111111111111
"$bin" app deploy --url "$url" --name demo --directory "$tmp/site" --commit "$sha" > "$tmp/first.json"
id1="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["id"])' "$tmp/first.json")"
curl -fsS "$url/store/apps/demo/index.html" > "$tmp/bootstrap"
grep -q "location.pathname.slice(1)" "$tmp/bootstrap"
test "$(curl -fsS "$url/store/apps/demo/releases/$id1/assets/app.js")" = "document.getElementById('app').textContent='HTTP release one';"
"$bin" app current --url "$url" --name demo | grep -q "$id1"
printf 'self.addEventListener("push",()=>{});' > "$tmp/worker.js"
"$bin" app publish-stable --url "$url" --name demo --asset sw.js   --file "$tmp/worker.js" > "$tmp/worker1.json"
test "$(curl -fsS "$url/store/apps/demo/sw.js")" =   'self.addEventListener("push",()=>{});'
"$bin" app publish-stable --url "$url" --name demo --asset sw.js   --file "$tmp/worker.js" > "$tmp/worker-repeat.json"
cmp "$tmp/worker1.json" "$tmp/worker-repeat.json"
printf 'self.addEventListener("push",()=>{ /* updated */ });' > "$tmp/worker.js"
"$bin" app publish-stable --url "$url" --name demo --asset sw.js   --file "$tmp/worker.js" > "$tmp/worker2.json"
test "$(curl -fsS "$url/store/apps/demo/sw.js")" =   'self.addEventListener("push",()=>{ /* updated */ });'
if "$bin" app publish-stable --url "$url" --name demo   --asset '../active.json' --file "$tmp/worker.js" >/dev/null 2>&1; then
  echo "stable publication accepted a traversal path" >&2; exit 1
fi
printf '{"start_url":"/","scope":"/"}' > "$tmp/manifest.webmanifest"
"$bin" app publish-stable --url "$url" --name demo --asset manifest.webmanifest   --file "$tmp/manifest.webmanifest" >/dev/null
test "$(curl -fsS "$url/store/apps/demo/manifest.webmanifest")" =   '{"start_url":"/","scope":"/"}'
printf '<svg xmlns="http://www.w3.org/2000/svg"></svg>' > "$tmp/icon.svg"
"$bin" app publish-stable --url "$url" --name demo --asset icon.svg   --file "$tmp/icon.svg" >/dev/null
test "$(curl -fsS "$url/store/apps/demo/icon.svg")" =   '<svg xmlns="http://www.w3.org/2000/svg"></svg>'
printf "document.getElementById('app').textContent='HTTP release two';" > "$tmp/site/assets/app.js"
"$bin" app deploy --url "$url" --name demo --directory "$tmp/site" --commit "$sha" > "$tmp/second.json"
id2="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["id"])' "$tmp/second.json")"
cmp "$tmp/bootstrap" <(curl -fsS "$url/store/apps/demo/index.html")
test "$id1" != "$id2"
"$bin" app versions --url "$url" --name demo > "$tmp/versions.json"
python3 - "$tmp/versions.json" <<'PY'
import json,sys
assert len(json.load(open(sys.argv[1]))) == 2
PY
"$bin" app verify --url "$url" --name demo --release "$id1"
"$bin" app rollback --url "$url" --name demo --release "$id1" > /dev/null
"$bin" app current --url "$url" --name demo | grep -q "$id1"
test "$(curl -fsS "$url/store/apps/demo/sw.js")" =   'self.addEventListener("push",()=>{ /* updated */ });'
"$bin" app rollback --url "$url" --name demo --release "$id2" > /dev/null
"$bin" app current --url "$url" --name demo | grep -q "$id2"
if "$bin" app rollback --url "$url" --name demo --release "00000000000000000000000000000000" > /dev/null 2>&1; then
  echo "incomplete release unexpectedly activated" >&2; exit 1
fi
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST -H "Authorization: Bearer $(cat "$tmp/root-token")" "$url/store/apps/demo" -d x)" = 409
echo "HTTP object releases, nested files, CAS rollback, history and verification passed"
origin_port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
origin="http://127.0.0.1:$origin_port"
python3 tests/http/fixture_app_origin.py "$url" "$origin_port" > "$tmp/origin.log" 2>&1 &
proxy_pid="$!"
for _ in $(seq 1 80); do
  if curl -fsS "$origin/active.json" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$proxy_pid" 2>/dev/null; then cat "$tmp/origin.log"; exit 1; fi
  sleep 0.1
done
test "$(curl -fsS -D "$tmp/headers" -o /dev/null "$origin/active.json" && grep -i 'Cache-Control: no-store' "$tmp/headers" | wc -l)" -ge 1
test "$(curl -s -o /dev/null -w '%{http_code}' "$origin/missing.js")" = 404
if command -v chromium >/dev/null 2>&1; then
  chromium --headless --no-sandbox --disable-gpu --disable-dev-shm-usage \
    --no-first-run --user-data-dir="$tmp/chromium" \
    --virtual-time-budget=5000 --dump-dom "$url/store/apps/demo/index.html" \
    > "$tmp/dom" 2> "$tmp/chrome-err" || true
  grep -q 'HTTP release two' "$tmp/dom" || {
    tail -24 "$tmp/chrome-err"; echo "browser static HTML bootstrap did not load"; exit 1;
  }
  echo "Chromium served nested relative assets through Skrynia HTTP"
  chromium --headless --no-sandbox --disable-gpu --disable-dev-shm-usage     --no-first-run --user-data-dir="$tmp/chromium-app"     --virtual-time-budget=5000 --dump-dom     "$origin/?some=query&argument=list#section" > "$tmp/app-dom" 2> "$tmp/app-err" || true
  grep -q 'HTTP release two' "$tmp/app-dom" || {
    tail -15 "$tmp/app-err"; echo "stable app origin did not load release"; exit 1;
  }
  grep -q "/releases/$id2/index.html?some=query&amp;argument=list#section" "$tmp/app-dom" || {
    echo "release navigation lost pathname, query, or fragment"; exit 1;
  }
  chromium --headless --no-sandbox --disable-gpu --disable-dev-shm-usage     --no-first-run --user-data-dir="$tmp/chromium-deep"     --virtual-time-budget=5000 --dump-dom     "$origin/docs/about.html?x=3#details" > "$tmp/deep-dom" 2> "$tmp/deep-err" || true
  grep -q 'Nested page works' "$tmp/deep-dom"
  grep -q "/releases/$id2/docs/about.html?x=3#details" "$tmp/deep-dom"
  echo "Chromium preserved app origin, relative assets, deep links, query and fragment"
fi
