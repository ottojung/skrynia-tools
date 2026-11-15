#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo"
python3 tests/browser/server.py --port 18799 > /tmp/skrynia-browser-server.log 2>&1 &
pid="$!"
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:18799/health >/dev/null; then break; fi
  sleep 0.2
done
chrome="$(command -v google-chrome || command -v chromium || command -v chromium-browser)"
output="$("$chrome" --headless=new --no-sandbox --disable-gpu --disable-dev-shm-usage \
  --virtual-time-budget=20000 --dump-dom \
  http://127.0.0.1:18799/tests/browser/index.html 2> /tmp/skrynia-browser-chrome.log)"
if ! grep -q 'data-result="pass"' <<<"$output"; then
  echo "Browser integration failed; HTML output follows:"
  echo "$output" | tail -35
  echo "Browser diagnostics:"
  tail -25 /tmp/skrynia-browser-chrome.log
  exit 1
fi
echo "Browser WebAssembly HTTP, CAS and log integration passed"
