#!/usr/bin/env bash
set -euo pipefail
umask 077
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
printf '%s\n' "${SKRYNIA_SSH_KEY:?missing SSH private key}" > "$tmp/key"
printf '%s\n' "${SKRYNIA_KNOWN_HOSTS:?missing SSH known-hosts}" > "$tmp/known_hosts"
chmod 600 "$tmp/key" "$tmp/known_hosts"
python3 - "$tmp/spec.json" <<'PY'
import json, os, re, sys
env = os.environ
host = env["SKRYNIA_HOST"]
user = env["SKRYNIA_USER"]
if not re.fullmatch(r"[A-Za-z0-9.-]+", host) or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_-]*", user):
    raise SystemExit("invalid SSH destination")
if not env["SKRYNIA_APP_NAME"] or not env["SKRYNIA_APPS_DIR"]:
    raise SystemExit("missing application name or hosting directory")
repo = env.get("SKRYNIA_REPO") or "https://github.com/" + env["GITHUB_REPOSITORY"]
commit = env.get("SKRYNIA_COMMIT") or (env.get("GITHUB_SHA") if not env.get("SKRYNIA_REPO") else None)
request = {"name": env["SKRYNIA_APP_NAME"], "apps_dir": env["SKRYNIA_APPS_DIR"],
           "repo": repo, "subdir": env.get("SKRYNIA_SUBDIR") or ".", "commit": commit,
           "builder_image": env.get("SKRYNIA_BUILDER_IMAGE") or "docker.io/library/rust:1.93-bookworm"}
with open(sys.argv[1], "w", encoding="utf8") as stream:
    json.dump(request, stream)
PY
ssh -T -o BatchMode=yes -o IdentitiesOnly=yes -o ForwardAgent=no \
  -o StrictHostKeyChecking=yes -o ConnectTimeout=15 \
  -o ServerAliveInterval=30 -o ServerAliveCountMax=4 \
  -o UserKnownHostsFile="$tmp/known_hosts" -i "$tmp/key" -- \
  "${SKRYNIA_USER}@${SKRYNIA_HOST}" \
  'skrynia-tools app deploy --spec-stdin true' < "$tmp/spec.json" > "$tmp/result.json"
python3 - "$tmp/result.json" <<'PY'
import json, os, re, sys
with open(sys.argv[1], encoding="utf8") as stream:
    result = json.load(stream)
name = result["name"]
release = result["id"]
commit = result["commit"]
if not re.fullmatch(r"[a-z0-9-]{1,64}", name):
    raise SystemExit("invalid release name")
if not re.fullmatch(r"[0-9a-f]{12}-[0-9a-f]{16}", release):
    raise SystemExit("invalid release ID")
if not re.fullmatch(r"[0-9a-f]{40}", commit):
    raise SystemExit("invalid commit SHA")
with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf8") as stream:
    stream.write(f"release={release}\ncommit={commit}\npath=/apps/{name}/\n")
PY
