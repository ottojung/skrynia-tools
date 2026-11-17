#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
directory="${SKRYNIA_BUILT_DIRECTORY:?build step missing}"
name="${SKRYNIA_APP_NAME:?missing app name}"
commit="${SKRYNIA_BUILT_COMMIT:?missing exact Git SHA}"
url="${SKRYNIA_URL:?missing Skrynia HTTP URL}"
token="${SKRYNIA_TOKEN:?missing scoped Skrynia user token}"
unset SKRYNIA_TOKEN
cargo build --locked --release --bin skrynia-tools   --manifest-path "$root/Cargo.toml"   --target-dir "${RUNNER_TEMP:?missing runner temp}/skrynia-tools-target"
binary="$RUNNER_TEMP/skrynia-tools-target/release/skrynia-tools"
release="$(SKRYNIA_TOKEN="$token" "$binary" app deploy --url "$url" --name "$name"    --directory "$directory" --commit "$commit"    --repo "${SKRYNIA_SOURCE_REPO:-https://github.com/$GITHUB_REPOSITORY}"    --subdir "${SKRYNIA_APP_SUBDIR:-.}")"
unset token
python3 -c '
import json,os,sys
r=json.loads(sys.argv[1])
name=os.environ["SKRYNIA_APP_NAME"]
if r["name"]!=name or r["commit"]!=os.environ["SKRYNIA_BUILT_COMMIT"]:
    raise SystemExit("released metadata does not match requested build")
prefix="apps/"+name+"/index.html"
url=os.environ["SKRYNIA_URL"].rstrip("/")+"/store/"+prefix
with open(os.environ["GITHUB_OUTPUT"],"a",encoding="utf8") as out:
    out.write("release="+r["id"]+"\ncommit="+r["commit"]+"\nurl="+url+"\n")
' "$release"
