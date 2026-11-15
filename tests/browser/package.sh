#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
out="$repo/dist/skrynia-wasm"
cp "$repo/browser-package/package.json" "$out/package.json"
cp "$repo/browser-package/README.md" "$out/README.md"
cp "$repo/LICENSE" "$out/LICENSE"
node --input-type=module -e '
import { readFileSync } from "node:fs";
const pkg = JSON.parse(readFileSync(process.argv[1]));
if (pkg.license !== "AGPL-3.0-or-later" ||
    pkg.exports["."].types !== "./skrynia_tools.d.ts" ||
    pkg.exports["."].import !== "./skrynia_tools.js") {
  throw new Error("invalid package metadata");
}' "$out/package.json"
node --input-type=module -e '
import { BrowserClient, sha256Hex } from "./dist/skrynia-wasm/skrynia_tools.js";
if (typeof BrowserClient !== "function" || typeof sha256Hex !== "function") {
  throw new Error("generated Wasm ESM exports unavailable");
}'
(cd "$out" && npm pack --dry-run --json) > "$out/pack.json"
python3 - "$out/pack.json" <<'PY'
import json, sys
files={v["path"] for v in json.load(open(sys.argv[1], encoding="utf8"))[0]["files"]}
required={"LICENSE","README.md","package.json","skrynia_tools.js",
          "skrynia_tools.d.ts","skrynia_tools_bg.wasm"}
assert required <= files, required - files
assert not any("secret" in path.lower() for path in files)
print("Installable npm browser package manifest validated")
PY
tarball="$(cd "$out" && npm pack --silent)"
test -f "$out/$tarball"
tar -tzf "$out/$tarball" | grep -q '^package/skrynia_tools_bg.wasm$'
echo "Browser npm package assembled: $tarball"
