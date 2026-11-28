#!/usr/bin/env bash
# One-time native-v3 trusted-host admin bootstrap. NOT a CLI, no subcommands.
# Supports local execution or one-time piped execution with an immutable pinned
# reviewed bundle and out-of-band checksum verification.
set -euo pipefail

umask 077
# The exact bootstrap bundle identity is the SHA-256 of bootstrap/SHA256SUMS,
# verified out-of-band (not self-referential: SHA256SUMS does not list
# itself). This is the only content identity passed to the installer.
REPO="${SKRYNIA_BOOTSTRAP_REPO:-ottojung/skrynia-tools}"

ask() {
  local name="$1" value="${!1-}"
  if [[ -z "$value" ]]; then
    read -r -p "$name: " value </dev/tty || return 2
  fi
  printf '%s' "$value"
}

verify_local_bundle() {
  local here="$1"
  local expected="${SKRYNIA_BOOTSTRAP_SOURCE_SHA256:-}"
  [[ -n "$expected" ]] || { echo 'SKRYNIA_BOOTSTRAP_SOURCE_SHA256 required (out-of-band immutable checksum of bootstrap/SHA256SUMS or trusted git checkout)' >&2; exit 2; }
  [[ -f "$here/SHA256SUMS" ]] || { echo 'SHA256SUMS missing; verify immutable bundle outside this tree' >&2; exit 2; }
  local manifest_actual
  manifest_actual="$(sha256sum "$here/SHA256SUMS" | awk '{print $1}')"
  [[ "$manifest_actual" == "$expected" ]] || { echo 'bootstrap SHA256SUMS checksum mismatch (out-of-band anchor required)' >&2; exit 2; }
  (cd "$here" && sha256sum -c SHA256SUMS >/dev/null) || { echo 'bootstrap SHA256SUMS verification failed' >&2; exit 2; }
  printf '%s' "$manifest_actual"
}

SOURCE_FILE="${BASH_SOURCE[0]:-}"
if [[ -z "$SOURCE_FILE" || "$SOURCE_FILE" == /dev/stdin || "$SOURCE_FILE" == - || ! -f "$SOURCE_FILE" ]]; then
  # One-time piped mode: immutable commit ref + out-of-band bundle checksum.
  # No self-referential hash, no mutable main fetch.
  REF="${SKRYNIA_BOOTSTRAP_SOURCE_REF:-}"
  [[ "$REF" =~ ^[0-9a-f]{40}$ ]] || { echo 'SKRYNIA_BOOTSTRAP_SOURCE_REF must be an immutable 40-hex reviewed commit' >&2; exit 2; }
  BUNDLE_SHA="${SKRYNIA_BOOTSTRAP_BUNDLE_SHA256:-}"
  [[ "$BUNDLE_SHA" =~ ^[0-9a-f]{64}$ ]] || { echo 'SKRYNIA_BOOTSTRAP_BUNDLE_SHA256 required (out-of-band sha256 of immutable tarball)' >&2; exit 2; }
  TMP="$(mktemp -d "${TMPDIR:-/tmp}/skrynia-bootstrap.XXXXXX")"
  trap 'rm -rf "$TMP"' EXIT
  TARBALL="$TMP/src.tar.gz"
  URL="https://codeload.github.com/${REPO}/tar.gz/${REF}"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --proto '=https' --tlsv1.2 "$URL" -o "$TARBALL"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$TARBALL" "$URL"
  else
    echo 'curl or wget required for piped bootstrap' >&2; exit 2
  fi
  ACTUAL="$(sha256sum "$TARBALL" | awk '{print $1}')"
  [[ "$ACTUAL" == "$BUNDLE_SHA" ]] || { echo 'immutable bootstrap bundle checksum mismatch' >&2; exit 2; }
  tar -xzf "$TARBALL" -C "$TMP"
  EXTRACTED="$(find "$TMP" -mindepth 1 -maxdepth 1 -type d | head -1)"
  [[ -f "$EXTRACTED/bootstrap/deploy.sh" ]] || { echo 'pinned bundle missing bootstrap/deploy.sh' >&2; exit 2; }
  # Out-of-band trust anchor is the bundle SHA256. The extracted SHA256SUMS is
  # verified inside the local bundle; pass its own hash only if operator supplied
  # it out-of-band, otherwise derive from the already-verified bundle manifest.
  export SKRYNIA_BOOTSTRAP_SOURCE_SHA256="${SKRYNIA_BOOTSTRAP_SOURCE_SHA256:-$(sha256sum "$EXTRACTED/bootstrap/SHA256SUMS" | awk '{print $1}')}"
  bash "$EXTRACTED/bootstrap/deploy.sh"
  exit $?
fi

HERE="$(cd -- "$(dirname -- "$SOURCE_FILE")" && pwd)"
manifest_actual="$(verify_local_bundle "$HERE")"

# Neither value may be overridden by the operator.
export SKRYNIA_BOOTSTRAP_BUNDLE_ID="$manifest_actual"
export SKRYNIA_ARTIFACT_ROOT="$(ask SKRYNIA_ARTIFACT_ROOT)"
export SKRYNIA_NGINX_ROOT="$(ask SKRYNIA_NGINX_ROOT)"
export SKRYNIA_PUBLICATION_MAILBOX="$(ask SKRYNIA_PUBLICATION_MAILBOX)"
export SKRYNIA_ORIGIN_URL="$(ask SKRYNIA_ORIGIN_URL)"
export SKRYNIA_NATIVE_READ_CAP_FILE="$(ask SKRYNIA_NATIVE_READ_CAP_FILE)"
export SKRYNIA_NATIVE_ADMIN_CAP_FILE="$(ask SKRYNIA_NATIVE_ADMIN_CAP_FILE)"
export SKRYNIA_NATIVE_ENDPOINT="${SKRYNIA_NATIVE_ENDPOINT:-/_skrynia/}"
export SKRYNIA_ADMIN_APP="${SKRYNIA_ADMIN_APP:-admin-v3}"
export SKRYNIA_PUBLISH_HELPER="$(ask SKRYNIA_PUBLISH_HELPER)"
export SKRYNIA_VERIFY_HELPER="$(ask SKRYNIA_VERIFY_HELPER)"

exec node "$HERE/install.mjs"
