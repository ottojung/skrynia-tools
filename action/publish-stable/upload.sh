#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
token="${SKRYNIA_TOKEN:?missing scoped grant token}"
unset SKRYNIA_TOKEN
target="${RUNNER_TEMP:?missing runner temp}/skrynia-tools-target"
binary="$target/release/skrynia-tools"
if [ ! -x "$binary" ]; then
  cargo build --locked --release --bin skrynia-tools \
    --manifest-path "$root/Cargo.toml" --target-dir "$target"
fi
SKRYNIA_TOKEN="$token" "$binary" app publish-stable \
  --url "${SKRYNIA_URL:?missing URL}" \
  --name "${SKRYNIA_APP_NAME:?missing app name}" \
  --asset "${SKRYNIA_STABLE_ASSET:?missing stable asset}" \
  --file "${SKRYNIA_STABLE_FILE:?missing file}"
unset token
