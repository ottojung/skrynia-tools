#!/usr/bin/env bash
# One-time native-v3 trusted-host admin bootstrap. NOT a CLI, no subcommands.
# Must be executed from a local file. Piped execution is explicitly unsupported.
set -euo pipefail

SOURCE_FILE="${BASH_SOURCE[0]}"
if [[ ! -f "$SOURCE_FILE" || "$SOURCE_FILE" == /dev/stdin || "$SOURCE_FILE" == - ]]; then
  echo 'refusing piped execution: download the pinned file, verify SHA-256, then run it locally' >&2
  exit 2
fi

# This installer MUST be run from a trusted local copy whose contents have
# been verified against an immutable source archive/checksum outside this
# tree (e.g. a signed SHA256SUMS published with the reviewed bootstrap
# release artifact). A self-referential commit pin inside the distributed
# files cannot be the source of truth. The operator must supply and confirm
# the reviewed bootstrap source checksum before execution.
EXPECTED_SOURCE_SHA256="${SKRYNIA_BOOTSTRAP_SOURCE_SHA256:-}"
if [[ -z "$EXPECTED_SOURCE_SHA256" ]]; then
  echo 'SKRYNIA_BOOTSTRAP_SOURCE_SHA256 is required: verify the immutable source archive/checksum outside this tree before running' >&2
  exit 2
fi
HERE="$(cd -- "$(dirname -- "$SOURCE_FILE")" && pwd)"
ROOT="$(cd -- "$HERE/.." && pwd)"
if [[ -f "$HERE/SHA256SUMS" ]]; then
  (cd "$HERE" && sha256sum -c SHA256SUMS >/dev/null) || { echo 'bootstrap SHA256SUMS verification failed' >&2; exit 2; }
  ACTUAL="$(sha256sum "$SOURCE_FILE" | awk '{print $1}')"
  [[ "$ACTUAL" == "$EXPECTED_SOURCE_SHA256" ]] || { echo 'bootstrap source checksum mismatch' >&2; exit 2; }
else
  echo 'SHA256SUMS missing: provide immutable source checksum verification outside this tree' >&2
  exit 2
fi

ask() {
  local name="$1" value="${!1-}"
  if [[ -z "$value" ]]; then
    read -r -p "$name: " value </dev/tty || return 2
  fi
  printf '%s' "$value"
}

# Kit artifacts are pinned to the reviewed tools commit; bootstrap source is
# pinned separately above. Neither value may be overridden by the operator.
SKRYNIA_TOOLS_PIN=639ec720da11ef311471535d6383ff556839a718
export SKRYNIA_TOOLS_PIN
export SKRYNIA_ARTIFACT_ROOT="$(ask SKRYNIA_ARTIFACT_ROOT)"
export SKRYNIA_ORIGIN_URL="$(ask SKRYNIA_ORIGIN_URL)"
export SKRYNIA_NATIVE_READ_CAP_FILE="$(ask SKRYNIA_NATIVE_READ_CAP_FILE)"
export SKRYNIA_NATIVE_ADMIN_CAP_FILE="$(ask SKRYNIA_NATIVE_ADMIN_CAP_FILE)"
export SKRYNIA_NATIVE_ENDPOINT="${SKRYNIA_NATIVE_ENDPOINT:-/_skrynia/}"
export SKRYNIA_ADMIN_APP="${SKRYNIA_ADMIN_APP:-admin-v3}"
export SKRYNIA_PUBLISH_HELPER="$(ask SKRYNIA_PUBLISH_HELPER)"
export SKRYNIA_VERIFY_HELPER="$(ask SKRYNIA_VERIFY_HELPER)"

exec node "$HERE/install.mjs"
