#!/usr/bin/env bash
# Real compiled P0 -> P1 -> P2 HTTP acceptance for the ordinary nonzero
# native-service P1. Test/benchmark only; not runtime application code.
# Uses PR53's dumb_p0_http.py fixture unchanged. All artifacts are pinned by
# sha256 so a rebuild or substitution cannot silently change the evidence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
KIT="/workspace/skrynia-v3-integration-kit"

KERNEL="${KERNEL:-$KIT/wasm-build-record/main-secure-pr122-20261121/skrynia-server}"
P0="${P0:-$KIT/artifacts/skrynia_program0_pr53_rebuild_d57ab27a.wasm}"
P1="${P1:-$KIT/artifacts/skrynia_native_service_p1clean_83a65a2f.wasm}"
ECHO="${ECHO:-$KIT/tests/guests/echo_guest.wasm}"

KERNEL_SHA=8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f
P0_SHA=d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa
P1_SHA=83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a
ECHO_SHA=dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5

for pair in "kernel:$KERNEL:$KERNEL_SHA" "p0:$P0:$P0_SHA" "p1:$P1:$P1_SHA" "echo:$ECHO:$ECHO_SHA"; do
  IFS=: read -r name path want <<<"$pair"
  test -f "$path" || { echo "MISSING $name: $path" >&2; exit 1; }
  got="$(sha256sum "$path" | cut -d' ' -f1)"
  test "$got" = "$want" || { echo "SHA MISMATCH $name: want $want got $got" >&2; exit 1; }
done

python3 "$ROOT/program0/integration/dumb_p0_http.py" \
  --kernel "$KERNEL" --p0 "$P0" --p1 "$P1" --echo-guest "$ECHO" \
  --kernel-sha256 "$KERNEL_SHA" --p0-sha256 "$P0_SHA" --p1-sha256 "$P1_SHA"
