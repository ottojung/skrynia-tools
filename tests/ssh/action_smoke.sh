#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/bin"
cat > "$tmp/bin/ssh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "$MOCK_SSH_ARGS"
cat > "$MOCK_SSH_INPUT"
if [[ "${MOCK_SSH_FAIL_STATUS:-}" != "" ]]; then exit "$MOCK_SSH_FAIL_STATUS"; fi
python3 - "$MOCK_SSH_INPUT" <<'PYSERVER'
import json, os, sys
with open(sys.argv[1], encoding="utf8") as f:
    req=json.load(f)
commit=req["commit"] or "0123456789abcdef0123456789abcdef01234567"
req["commit"]=commit
req["id"]=f"{commit[:12]}-0123456789abcdef"
if os.getenv("MOCK_OVERRIDE_NAME"):
    req["name"]=os.environ["MOCK_OVERRIDE_NAME"]
if os.getenv("MOCK_OVERRIDE_COMMIT"):
    req["commit"]=os.environ["MOCK_OVERRIDE_COMMIT"]
if os.getenv("MOCK_OVERRIDE_REPO"):
    req["repo"]=os.environ["MOCK_OVERRIDE_REPO"]
print(json.dumps(req))
PYSERVER
SH
chmod +x "$tmp/bin/ssh"
export PATH="$tmp/bin:$PATH" MOCK_SSH_INPUT="$tmp/spec.json"
export MOCK_SSH_ARGS="$tmp/ssh-args" SKRYNIA_SSH_PORT=2222
export GITHUB_OUTPUT="$tmp/github-output"
export GITHUB_REPOSITORY="ottojung/sample" GITHUB_SHA="0123456789abcdef0123456789abcdef01234567"
export SKRYNIA_HOST=example.org SKRYNIA_USER=deployer
export SKRYNIA_SSH_KEY="fake test private key" SKRYNIA_KNOWN_HOSTS="example.org ssh-ed25519 fake"
export SKRYNIA_APP_NAME=example SKRYNIA_APPS_DIR=/srv/apps SKRYNIA_REPO=""
export SKRYNIA_SUBDIR=apps/example SKRYNIA_COMMIT="" SKRYNIA_BUILDER_IMAGE=""
bash "$repo/action/deploy/deploy.sh"
python3 - "$tmp/spec.json" "$tmp/github-output" <<'PY'
import json, sys
spec=json.load(open(sys.argv[1],encoding="utf8"))
assert spec["repo"]=="https://github.com/ottojung/sample"
assert spec["commit"]=="0123456789abcdef0123456789abcdef01234567"
assert spec["subdir"]=="apps/example"
assert "release=0123456789ab-0123456789abcdef" in open(sys.argv[2]).read()
PY
grep -qx -- '-F' "$tmp/ssh-args"
grep -qx -- '/dev/null' "$tmp/ssh-args"
grep -qx -- '2222' "$tmp/ssh-args"
export SKRYNIA_REPO=https://github.com/other/repository
bash "$repo/action/deploy/deploy.sh"
python3 - "$tmp/spec.json" <<'PY'
import json, sys
assert json.load(open(sys.argv[1]))["commit"] is None
PY

export SKRYNIA_REPO=""
original_output="$(cat "$tmp/github-output")"
for field in MOCK_OVERRIDE_NAME MOCK_OVERRIDE_COMMIT MOCK_OVERRIDE_REPO; do
  case "$field" in
    MOCK_OVERRIDE_NAME) value="wrong-name" ;;
    MOCK_OVERRIDE_COMMIT) value="ffffffffffffffffffffffffffffffffffffffff" ;;
    MOCK_OVERRIDE_REPO) value="https://github.com/other/replaced" ;;
  esac
  export "$field=$value"
  if bash "$repo/action/deploy/deploy.sh" > "$tmp/unexpected.log" 2>&1; then
    echo "ERROR: remote result mismatch accepted: $field" >&2
    exit 1
  fi
  unset "$field"
  test "$(cat "$tmp/github-output")" = "$original_output"
done
echo "SSH deploy composite-action provenance/refusal checks passed"

export MOCK_SSH_FAIL_STATUS=124
if bash "$repo/action/deploy/deploy.sh" > "$tmp/timedout.log" 2>&1; then
  echo 'ERROR: lost SSH result treated as committed deployment' >&2
  exit 1
fi
grep -q 'remote activation may have occurred' "$tmp/timedout.log"
test "$(cat "$tmp/github-output")" = "$original_output"
unset MOCK_SSH_FAIL_STATUS
echo "SSH timeout/lost-result handling checks passed"
