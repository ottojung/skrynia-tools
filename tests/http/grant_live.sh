#!/usr/bin/env bash
set -euo pipefail
umask 077
server="$SKRYNIA_TEST_SERVER_BIN"
tool="$SKRYNIA_TEST_CLI_BIN"
tmp="$(mktemp -d)"
pid=""
cleanup() {
  if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; fi
  rm -rf "$tmp"
}
trap cleanup EXIT
mkdir "$tmp/root"
port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
url="http://127.0.0.1:$port"
SKRYNIA_DATA_DIR="$tmp/root" SKRYNIA_PORT="$port" SKRYNIA_HOST=127.0.0.1 \
  SKRYNIA_URL="$url" "$server" > "$tmp/server.log" 2>&1 &
pid="$!"
ready=0
for _ in $(seq 1 80); do
  if curl -fsS "$url/health" >/dev/null 2>&1; then ready=1; break; fi
  if ! kill -0 "$pid" 2>/dev/null; then break; fi
  sleep 0.1
done
if [ "$ready" != 1 ]; then cat "$tmp/server.log"; exit 1; fi
if env -u SKRYNIA_DATA_DIR SKRYNIA_URL="$url" "$tool" admin grant \
  --parent-grant-is-root --locations 'apps/*' --max-bytes 5000000 \
  --max-objects 200 --credentials-out "$tmp/unconfigured.json" >"$tmp/error" 2>&1; then
  echo "--parent-grant-is-root succeeded without SKRYNIA_DATA_DIR" >&2; exit 1
fi
grep -q SKRYNIA_DATA_DIR "$tmp/error"
test ! -e "$tmp/unconfigured.json"
if SKRYNIA_URL="$url" SKRYNIA_DATA_DIR="$tmp/missing-deployment" \
  "$tool" admin grant --parent-grant-is-root --locations 'apps/*' --max-bytes 6000 \
  --max-objects 4 --credentials-out "$tmp/missing-root.json" >/dev/null 2>&1; then
  echo "accepted nonexistent Skrynia data directory" >&2; exit 1
fi
test ! -e "$tmp/missing-root.json"
SKRYNIA_URL="$url" SKRYNIA_PARENT_GRANT=not-a-token SKRYNIA_DATA_DIR="$tmp/root" "$tool" admin grant \
  --parent-grant-is-root --locations 'apps/*' \
  --operations create,modify,list,delegate \
  --max-bytes 5000000 --max-objects 200 \
  --credentials-out "$tmp/parent.json" > "$tmp/parent-result.json"
python3 - "$tmp/parent.json" "$tmp/parent-result.json" <<'PY'
import json,os,sys
record=json.load(open(sys.argv[1],encoding="utf8"))
result=json.load(open(sys.argv[2],encoding="utf8"))
assert record["grant"]["id"]==result["grantId"]
assert record["grant"]["locations"]=="apps/*"
assert set(record["grant"]["operations"])=={"create","modify","list","delegate"}
assert record["grant"]["issuerToken"] and record["user"]["userToken"]
assert os.stat(sys.argv[1]).st_mode & 0o077 == 0
assert "userToken" not in result and "issuerToken" not in result
PY
parent_token="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["user"]["userToken"])' "$tmp/parent.json")"
SKRYNIA_URL=http://127.0.0.1:1 SKRYNIA_PARENT_GRANT=not-a-token \
  "$tool" admin grant --url "$url" \
  --parent-grant "$parent_token" --locations 'apps/long/*' \
  --max-bytes 50000 --max-objects 20 \
  --credentials-out "$tmp/child.json" > "$tmp/child-result.json"
python3 - "$tmp/child.json" <<'PY'
import json,sys
record=json.load(open(sys.argv[1],encoding="utf8"))
assert record["grant"]["locations"]=="apps/long/*"
assert set(record["grant"]["operations"])=={"create","modify","list"}
PY
SKRYNIA_URL="$url" "$tool" admin grant --existing-grant "$tmp/child.json" \
  --credentials-out "$tmp/reissued.json" > "$tmp/reissued-result.json"
python3 - "$tmp/child.json" "$tmp/reissued.json" <<'PY'
import json,sys
a,b=[json.load(open(p,encoding="utf8")) for p in sys.argv[1:]]
assert a["grant"]["id"]==b["grant"]["id"]
assert a["user"]["userToken"]!=b["user"]["userToken"]
PY
token="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["user"]["userToken"])' "$tmp/child.json")"
status="$(curl -sS -o /dev/null -w '%{http_code}' \
  -X POST -H "Authorization: Bearer $token" \
  "$url/store/apps/long/test.js" --data-binary 'worked')"
test "$status" = 201
status="$(curl -sS -o /dev/null -w '%{http_code}' \
  -X POST -H "Authorization: Bearer $token" \
  "$url/store/apps/other/test.js" --data-binary 'forbidden')"
test "$status" = 403
# A second CLI family must honor SKRYNIA_URL, not only admin commands.
SKRYNIA_URL="$url" "$tool" object get --key apps/long/test.js > "$tmp/object.txt"
test "$(cat "$tmp/object.txt")" = worked
# Omitted --credentials-out uses a private randomized XDG state path.
XDG_STATE_HOME="$tmp/state" SKRYNIA_URL="$url" \
  SKRYNIA_PARENT_GRANT="$parent_token" "$tool" admin grant \
  --locations 'apps/tesicca/*' --max-bytes 9000 --max-objects 15 \
  > "$tmp/default-result.json"
python3 - "$tmp/default-result.json" "$tmp/state" <<'PYXD'
import json,os,pathlib,sys
info=json.load(open(sys.argv[1],encoding="utf8"))
path=pathlib.Path(info["credentialsFile"])
root=pathlib.Path(sys.argv[2])
assert path.parent==root/"skrynia-tools"/"grants"
assert path.exists() and path.suffix==".json"
assert os.stat(path).st_mode & 0o077==0
assert os.stat(path.parent).st_mode & 0o077==0
assert json.load(open(path,encoding="utf8"))["grant"]["locations"]=="apps/tesicca/*"
PYXD
if SKRYNIA_URL="$url" "$tool" admin grant --existing-grant "$tmp/child.json" \
  --locations 'apps/other/*' --credentials-out "$tmp/conflicting.json" \
  >/dev/null 2>&1; then
  echo "accepted conflicting existing-grant options" >&2; exit 1
fi
test ! -e "$tmp/conflicting.json"
if SKRYNIA_URL="$url" "$tool" admin grant --parent-grant "$parent_token" \
  --locations 'apps/bad/*' --operations make-everything --max-bytes 10 \
  --max-objects 10 --credentials-out "$tmp/invalid.json" >/dev/null 2>&1; then
  echo "invalid grant operations accepted" >&2; exit 1
fi
test ! -e "$tmp/invalid.json"
# Parent grant is a bearer token, never a path to saved credentials.
if SKRYNIA_URL="$url" "$tool" admin grant \
  --parent-grant "$tmp/parent.json" --locations 'apps/path/*' \
  --max-bytes 6000 --max-objects 4 --credentials-out "$tmp/path-arg.json" \
  >/dev/null 2>&1; then
  echo "accepted a path instead of parent grant token" >&2; exit 1
fi
test ! -e "$tmp/path-arg.json"
# Explicitly choosing the root conflicts with another explicit parent token.
if SKRYNIA_URL="$url" SKRYNIA_DATA_DIR="$tmp/root" "$tool" admin grant \
  --parent-grant-is-root --parent-grant "$parent_token" \
  --locations 'apps/*' --max-bytes 6000 --max-objects 4 \
  --credentials-out "$tmp/both-parents.json" >/dev/null 2>&1; then
  echo "accepted two parents" >&2; exit 1
fi
test ! -e "$tmp/both-parents.json"
# Root grant must reject alternate configuration instead of silently using it.
if SKRYNIA_URL="$url" SKRYNIA_DATA_DIR="$tmp/root" "$tool" admin grant \
  --parent-grant-is-root --data-dir "$tmp/root" --locations 'apps/*' \
  --max-bytes 6000 --max-objects 4 --credentials-out "$tmp/wrong-flag.json" \
  > "$tmp/wrong-flag.log" 2>&1; then
  echo "accepted a separate root-data-dir setting" >&2; exit 1
fi
grep -q SKRYNIA_DATA_DIR "$tmp/wrong-flag.log"
test ! -e "$tmp/wrong-flag.json"
echo "PASS: generic grant, SKRYNIA_URL fallback/override, SKRYNIA_DATA_DIR root, parent, reuse, HTTP isolation"
