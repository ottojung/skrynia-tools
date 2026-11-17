#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/workspace/app"
cat > "$tmp/workspace/app/Makefile" <<'MAKE'
build:
	mkdir -p dist/assets
	printf '<!doctype html><script src="./assets/app.js"></script>' > dist/index.html
	printf 'console.log("ok")' > dist/assets/app.js
MAKE
git -C "$tmp/workspace" init -q
git -C "$tmp/workspace" config user.email test@example.com
git -C "$tmp/workspace" config user.name Test
git -C "$tmp/workspace" add .
git -C "$tmp/workspace" commit -qm initial
commit="$(git -C "$tmp/workspace" rev-parse HEAD)"
export GITHUB_WORKSPACE="$tmp/workspace" GITHUB_REPOSITORY="owner/repo"
export GITHUB_SHA="$commit" GITHUB_OUTPUT="$tmp/github-output"
export SKRYNIA_APP_NAME=test SKRYNIA_APP_SUBDIR=app SKRYNIA_BUILD_OUTPUT=dist
export SKRYNIA_SOURCE_REPO="" SKRYNIA_SOURCE_COMMIT="" SKRYNIA_BUILDER_IMAGE=""
bash "$repo/action/deploy/build.sh"
grep -Fq "directory=$tmp/workspace/app/dist" "$GITHUB_OUTPUT"
grep -Fq "commit=$commit" "$GITHUB_OUTPUT"
test -f "$tmp/workspace/app/dist/assets/app.js"
export SKRYNIA_SOURCE_COMMIT=ffffffffffffffffffffffffffffffffffffffff
if bash "$repo/action/deploy/build.sh" > "$tmp/failed" 2>&1; then
  echo "Action built a different commit than requested" >&2; exit 1
fi
export SKRYNIA_SOURCE_COMMIT=""
export SKRYNIA_APP_SUBDIR=../private
if bash "$repo/action/deploy/build.sh" > "$tmp/failed" 2>&1; then
  echo "Action accepted traversal in source subdir" >&2; exit 1
fi
echo "HTTP-only Action builds Git commit locally and refuses mismatch/traversal"
