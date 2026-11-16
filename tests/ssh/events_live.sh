#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
server="${SKRYNIA_TEST_SERVER_BIN:?provide the built skrynia-server binary}"
tmp="$(mktemp -d /tmp/skrynia-events-live-XXXXXX)"
pid=""
cleanup() {
  if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi
  rm -rf "$tmp"
}
trap cleanup EXIT
mkdir "$tmp/state"
port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
export SKRYNIA_DATA_DIR="$tmp/state" SKRYNIA_URL="http://127.0.0.1:$port"
SKRYNIA_HOST=127.0.0.1 SKRYNIA_PORT="$port" "$server" > "$tmp/server.log" 2>&1 &
pid="$!"
ready=0
for _ in $(seq 1 60); do
  if curl -sS --connect-timeout 1 -o /dev/null "$SKRYNIA_URL/store/missing" 2>/dev/null; then
    ready=1
    break
  fi
  if ! kill -0 "$pid" 2>/dev/null; then break; fi
  sleep .1
done
if [ "$ready" != 1 ]; then cat "$tmp/server.log"; exit 1; fi
"$server" root-credentials > "$tmp/root.json"
python3 - "$tmp/root.json" "$SKRYNIA_URL" "$tmp/token" <<'PY'
import json,sys,urllib.request
root=json.load(open(sys.argv[1],encoding="utf8"))
url=sys.argv[2]
def post(path,token,body):
  request=urllib.request.Request(url+path,data=json.dumps(body).encode(),
      method="POST",headers={"Authorization":"Bearer "+token,"Content-Type":"application/json"})
  with urllib.request.urlopen(request,timeout=10) as response:
    return json.load(response)
parent=post("/grants/"+root["id"]+"/users",root["issuerToken"],{})
grant=post("/grants",parent["userToken"],{"locations":"events.*",
    "operations":["create","list"],"maxBytes":1000000,"maxObjects":100})
user=post("/grants/"+grant["id"]+"/users",grant["issuerToken"],{})
with open(sys.argv[3],"w",encoding="utf8") as f:
  f.write(user["userToken"])
PY
SKRYNIA_EVENT_TOKEN="$(cat "$tmp/token")" SKRYNIA_EVENT_PREFIX="events.live" \
  cargo run --quiet --manifest-path "$repo/Cargo.toml" --example event_set
echo "Real Skrynia HTTP event-set integration passed"
