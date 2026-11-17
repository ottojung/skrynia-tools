#!/usr/bin/env bash
set -euo pipefail
# One-time bootstrap, not a grant-administration CLI.
ask() {
  local var="$1" question="$2" secret="$3" value=""
  if [[ -n "${!var:-}" ]]; then return; fi
  if [[ ! -r /dev/tty ]]; then
    printf 'Missing %s; set it in the environment (no terminal).\n' "$var" >&2
    exit 2
  fi
  if [[ "$secret" == yes ]]; then
    printf '%s: ' "$question" >/dev/tty
    IFS= read -r -s value </dev/tty || exit 2
    printf '\n' >/dev/tty
  else
    IFS= read -r -p "$question: " value </dev/tty || exit 2
  fi
  if [[ -z "$value" ]]; then
    printf '%s is required.\n' "$var" >&2
    exit 2
  fi
  printf -v "$var" '%s' "$value"
  export "$var"
}
ask SKRYNIA_URL "Skrynia HTTPS URL" no
ask SKRYNIA_TOOLS_GRANT "Grant user token for apps scope" yes
ask SKRYNIA_TOOLS_ORIGIN "Dedicated admin HTTPS origin (e.g. https://tools.a.vau.place)" no
command -v node >/dev/null || { echo 'Node.js 22+ is required.' >&2; exit 2; }
node -e "process.exit(Number(process.versions.node.split('.')[0]) >= 22 ? 0 : 1)" || {
  echo 'Node.js 22+ is required.' >&2
  exit 2
}
command -v curl >/dev/null || { echo 'curl is required.' >&2; exit 2; }
# Never replace an encrypted vault with a fresh password silently.
app="${SKRYNIA_TOOLS_APP:-tools}"
if curl -fsS "${SKRYNIA_URL%/}/store/apps/$app/active.json" >/dev/null 2>&1; then
  ask SKRYNIA_TOOLS_ACCESS_SECRET "Existing access secret (upgrade)" yes
fi
source_dir="${SKRYNIA_TOOLS_SOURCE_DIR:-}"
tmp=""
if [[ -z "$source_dir" ]]; then
  command -v tar >/dev/null || { echo 'tar is required.' >&2; exit 2; }
  ref="${SKRYNIA_TOOLS_REF:-main}"
  if [[ ! "$ref" =~ ^[a-zA-Z0-9._-]+$ ]]; then
    echo 'Invalid SKRYNIA_TOOLS_REF' >&2
    exit 2
  fi
  tmp="$(mktemp -d)"
  trap 'rm -rf -- "$tmp"' EXIT
  curl --fail --silent --show-error --location \
    "https://github.com/ottojung/skrynia-tools/archive/${ref}.tar.gz" \
    | tar xz -C "$tmp" --strip-components=1
  source_dir="$tmp"
fi
test -f "$source_dir/bootstrap/deploy.mjs" || {
  echo 'Source tree does not contain Skrynia Tools web administration.' >&2
  exit 2
}
node "$source_dir/bootstrap/deploy.mjs" "$source_dir"
