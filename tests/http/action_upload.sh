#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/build"
printf 'app' > "$tmp/build/index.html"
cat > "$tmp/bin/cargo" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
test -z "${SKRYNIA_TOKEN:-}" || exit 95
mkdir -p "$RUNNER_TEMP/skrynia-tools-target/release"
cat > "$RUNNER_TEMP/skrynia-tools-target/release/skrynia-tools" <<'CLI'
#!/usr/bin/env bash
test "$SKRYNIA_TOKEN" = 'test-scoped-secret' || exit 96
echo '{"name":"my-app","commit":"0123456789012345678901234567890123456789","id":"0123456789abcdef0123456789abcdef"}'
CLI
chmod +x "$RUNNER_TEMP/skrynia-tools-target/release/skrynia-tools"
MOCK
chmod +x "$tmp/bin/cargo"
export PATH="$tmp/bin:$PATH" RUNNER_TEMP="$tmp"
export SKRYNIA_BUILT_DIRECTORY="$tmp/build" SKRYNIA_APP_NAME=my-app
export SKRYNIA_BUILT_COMMIT=0123456789012345678901234567890123456789
export SKRYNIA_URL=https://example.org/skrynia SKRYNIA_TOKEN=test-scoped-secret
export GITHUB_REPOSITORY=ottojung/app GITHUB_OUTPUT="$tmp/output"
bash "$repo/action/deploy/upload.sh"
grep -Fqx 'url=https://example.org/skrynia/store/apps/my-app/index.html' "$tmp/output"
grep -Fqx 'release=0123456789abcdef0123456789abcdef' "$tmp/output"
