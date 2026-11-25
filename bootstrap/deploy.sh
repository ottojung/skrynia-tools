#!/usr/bin/env bash
# One-time native-v3 trusted-host admin bootstrap. NOT a CLI, no subcommands.
# Supports local execution or one-time piped execution with an immutable pinned
# reviewed bundle and out-of-band checksum verification.
set -euo pipefail

umask 077
TOOLS_PIN=639ec720da11ef311471535d6383ff556839a718
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
  [[ -n "$expected" ]] || { echo 'SKRYNIA_BOOTSTRAP_SOURCE_SHA256 required (out-of-band immutable checksum)' >&2; exit 2; }
  [[ -f "$here/SHA256SUMS" ]] || { echo 'SHA256SUMS missing; verify immutable bundle outside this tree' >&2; exit 2; }
  (cd "$here" && sha256sum -c SHA256SUMS >/dev/null) || { echo 'bootstrap SHA256SUMS verification failed' >&2; exit 2; }
  local actual
  actual="$(sha256sum "$here/deploy.sh" | awk '{print $1}')"
  [[ "$actual" == "$expected" ]] || { echo 'bootstrap source checksum mismatch' >&2; exit 2; }
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
  export SKRYNIA_BOOTSTRAP_SOURCE_SHA256="${SKRYNIA_BOOTSTRAP_SOURCE_SHA256:-$(sha256sum "$EXTRACTED/bootstrap/deploy.sh" | awk '{print $1}')}"
  exec bash "$EXTRACTED/bootstrap/deploy.sh"
fi

HERE="$(cd -- "$(dirname -- "$SOURCE_FILE")" && pwd)"
verify_local_bundle "$HERE"

# Neither value may be overridden by the operator.
export SKRYNIA_TOOLS_PIN="$TOOLS_PIN"
export SKRYNIA_ARTIFACT_ROOT="$(ask SKRYNIA_ARTIFACT_ROOT)"
export SKRYNIA_ORIGIN_URL="$(ask SKRYNIA_ORIGIN_URL)"
export SKRYNIA_NATIVE_READ_CAP_FILE="$(ask SKRYNIA_NATIVE_READ_CAP_FILE)"
export SKRYNIA_NATIVE_ADMIN_CAP_FILE="$(ask SKRYNIA_NATIVE_ADMIN_CAP_FILE)"
export SKRYNIA_NATIVE_ENDPOINT="${SKRYNIA_NATIVE_ENDPOINT:-/_skrynia/}"
export SKRYNIA_ADMIN_APP="${SKRYNIA_ADMIN_APP:-admin-v3}"
export SKRYNIA_PUBLISH_HELPER="$(ask SKRYNIA_PUBLISH_HELPER)"
export SKRYNIA_VERIFY_HELPER="$(ask SKRYNIA_VERIFY_HELPER)"

exec node "$HERE/install.mjs"
