#!/usr/bin/env bash
# Real target-ABI acceptance with explicit artifact pins, never production.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
: "${KERNEL:?Set KERNEL to the compiled target kernel}"
: "${P0:?Set P0 to the compiled Program 0}"
: "${P1:?Set P1 to the compiled ordinary service}"
: "${P2:?Set P2 to the compiled Theorem Lab guest}"
: "${KERNEL_SHA:?Set the expected KERNEL SHA256}"
: "${P0_SHA:?Set the expected P0 SHA256}"
: "${P1_SHA:?Set the expected P1 SHA256}"
: "${P2_SHA:?Set the expected P2 SHA256}"
for name in KERNEL P0 P1 P2; do
  path="${!name}"
  pin="${name}_SHA"
  test -f "$path"
  got="$(sha256sum "$path" | cut -d' ' -f1)"
  test "$got" = "${!pin}" || { echo "SHA mismatch: $name" >&2; exit 1; }
done
python3 "$ROOT/native-service/integration/verify_target_wasm.py" "$P1"
test "$(wc -c < "$P1")" -lt 2096000
python3 "$ROOT/tests/v3_proxy/registry_recovery.py" \
  --kernel "$KERNEL" --p0 "$P0" --p1 "$P1" --guest "$P2"
python3 "$ROOT/examples/theorem_lab_feedback_p2/tests/real_kit_p2.py" \
  --kernel "$KERNEL" --p0 "$P0" --p1 "$P1" --p2 "$P2" \
  --kernel-sha256 "$KERNEL_SHA" --p0-sha256 "$P0_SHA" \
  --p1-sha256 "$P1_SHA" --p2-sha256 "$P2_SHA"
