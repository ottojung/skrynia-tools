#!/usr/bin/env bash
set -euo pipefail
name="${SKRYNIA_APP_NAME:?missing app name}"
subdir="${SKRYNIA_APP_SUBDIR:-.}"
output="${SKRYNIA_BUILD_OUTPUT:-build}"
repo="${SKRYNIA_SOURCE_REPO:-}"
commit="${SKRYNIA_SOURCE_COMMIT:-${GITHUB_SHA:?missing commit}}"
[[ "$name" =~ ^[a-z0-9][a-z0-9-]{0,63}$ ]] || exit 2
[[ "$commit" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "$subdir" != /* && "$output" != /* ]] || exit 2
[[ "/$subdir/$output/" != *"/../"* ]] || exit 2
[[ "/$subdir/$output/" != *"/./../"* ]] || exit 2
source_root="${GITHUB_WORKSPACE:?missing GitHub workspace}"
if [[ -n "$repo" && "$repo" != "https://github.com/$GITHUB_REPOSITORY" ]]; then
  [[ "$repo" =~ ^https://github.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(.git)?$ ]] || exit 2
  source_root="$(mktemp -d)"
  git clone --quiet --no-checkout "$repo" "$source_root"
  git -C "$source_root" checkout --quiet --detach "$commit"
else
  [[ "$(git -C "$source_root" rev-parse HEAD)" == "$commit" ]] || {
    echo "GitHub checkout differs from requested commit" >&2; exit 2;
  }
fi
[[ -f "$source_root/$subdir/Makefile" ]] || { echo "No application Makefile" >&2; exit 2; }
builder="${SKRYNIA_BUILDER_IMAGE:-}"
if [[ -n "$builder" ]]; then
  [[ "$builder" != -* && "$builder" =~ ^[A-Za-z0-9_./:@-]+$ ]] || exit 2
  docker run --rm --user "$(id -u):$(id -g)" --cap-drop ALL --security-opt no-new-privileges     -v "$source_root:/source" -w "/source/$subdir" "$builder" make build
else
  make -C "$source_root/$subdir" build
fi
directory="$source_root/$subdir/$output"
[[ -f "$directory/index.html" ]] || { echo "No built index.html" >&2; exit 2; }
{
  echo "directory=$directory"
  echo "commit=$commit"
} >> "${GITHUB_OUTPUT:?missing GitHub output}"
