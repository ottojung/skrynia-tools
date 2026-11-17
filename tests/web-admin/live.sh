#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
server="${SKRYNIA_TEST_SERVER_BIN:?Need Skrynia Rust server}"
tmp="$(mktemp -d)"
pid=""
proxy_pid=""
cleanup() {
  if [[ -n "$proxy_pid" ]]; then kill "$proxy_pid" 2>/dev/null || :; wait "$proxy_pid" 2>/dev/null || :; fi
  if [[ -n "$pid" ]]; then kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; fi
  rm -rf "$tmp"
}
trap cleanup EXIT
mkdir "$tmp/root"
port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
web_port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
base="http://127.0.0.1:$port"
origin="http://127.0.0.1:$web_port"
SKRYNIA_URL="$base/_skrynia" SKRYNIA_DATA_DIR="$tmp/root" SKRYNIA_HOST=127.0.0.1 \
  SKRYNIA_PORT="$port" "$server" > "$tmp/server.log" 2>&1 &
pid="$!"
for i in {1..80}; do
  if curl -fsS "$base/_skrynia/health" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$pid" 2>/dev/null; then cat "$tmp/server.log"; exit 1; fi
  sleep .1
done
SKRYNIA_DATA_DIR="$tmp/root" "$server" root-credentials > "$tmp/root.json"
python3 - "$base/_skrynia" "$tmp/root.json" "$tmp/token" <<'PY'
import json,sys,urllib.request
url,rootfile,out=sys.argv[1:]
root=json.load(open(rootfile,encoding="utf8"))
def post(path,token,body):
  req=urllib.request.Request(url+path,method="POST",
    data=json.dumps(body).encode(),headers={
      "Content-Type":"application/json","Authorization":"Bearer "+token})
  with urllib.request.urlopen(req,timeout=15) as reply:
    return json.load(reply)
parent=post("/grants/"+root["id"]+"/users",root["issuerToken"],{})["userToken"]
grant=post("/grants",parent,{"locations":"apps/*",
  "operations":["create","modify","list","delegate"],
  "maxBytes":100000000,"maxObjects":500})
child=post("/grants/"+grant["id"]+"/users",grant["issuerToken"],{})
with open(out,"w",encoding="utf8") as file:
  file.write(child["userToken"])
PY
node "$repo/tests/web-admin/app-host.mjs" "$base" "$web_port" tools \
  > "$tmp/proxy.log" 2>&1 &
proxy_pid="$!"
for i in {1..80}; do
  if curl -fsS "$origin/_skrynia/health" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$proxy_pid" 2>/dev/null; then cat "$tmp/proxy.log"; exit 1; fi
  sleep .1
done
export SKRYNIA_URL="$base/_skrynia"
export SKRYNIA_TOOLS_GRANT="$(cat "$tmp/token")"
export SKRYNIA_TOOLS_ORIGIN="$origin"
export SKRYNIA_TOOLS_SOURCE_DIR="$repo"
bash "$repo/deploy.sh" > "$tmp/output"
password="$(awk '/^Access secret:/ {print $3}' "$tmp/output")"
test "${#password}" -ge 40
node "$repo/tests/web-admin/releases.mjs"
if [[ "${SKRYNIA_TEST_BROWSER:-0}" == 1 ]]; then
  TEST_ACCESS_SECRET="$password" node "$repo/tests/web-admin/browser.mjs"
fi
TEST_ACCESS_SECRET="$password" node "$repo/tests/web-admin/verify.mjs"
# Upgrade must preserve the same access secret and encrypted vault.
prior_id="$(curl -fsS "$origin/active.json" | python3 -c 'import json,sys;print(json.load(sys.stdin)["release"])')"
export SKRYNIA_TOOLS_ACCESS_SECRET="$password"
bash "$repo/deploy.sh" > "$tmp/upgrade-output"
test "$(awk '/^Access secret:/ {print $3}' "$tmp/upgrade-output")" = "$password"
next_id="$(curl -fsS "$origin/active.json" | python3 -c 'import json,sys;print(json.load(sys.stdin)["release"])')"
test "$prior_id" != "$next_id"
TEST_ACCESS_SECRET="$password" node "$repo/tests/web-admin/verify.mjs"
echo "Skrynia web admin: disposable real-server bootstrap, safe upgrade and grant lifecycle passed"
