# Ordinary service target-ABI integration

Program 0 authenticates and proxies bytes; ordinary P1 owns the registry,
appdata and userland mutation policy. Its P2 interpreter supplies a single
framed callback in P1's own execution context. See
[the P1 port](../../docs/target-t3-native-service-port.md) and
[the P2 interpreter](../../docs/p2-interpreter.md).

## Build and verify

Use Rust 1.93.0 with the wasm32-unknown-unknown target and the locked
dependencies. From the tools checkout:

    export RUSTFLAGS='-C link-arg=--export-table -C opt-level=z -C strip=symbols'
    cargo +1.93.0 build --locked --release --target wasm32-unknown-unknown \
      -p skrynia-program0 -p skrynia-native-service --features skrynia-native-service/cb-guest
    python3 native-service/integration/verify_target_wasm.py \
      target/wasm32-unknown-unknown/release/skrynia_native_service.wasm
    test "$(wc -c < target/wasm32-unknown-unknown/release/skrynia_native_service.wasm)" -lt 2096000

The size check enforces the current P0 slot's code capacity. An ordinary
unoptimized release build may be too large; use the flags above.

## Real acceptance

Supply the paths of the compiled kernel, P0, P1 and Theorem Lab P2 via
KERNEL, P0, P1 and P2, with their expected SHA256 digests via KERNEL_SHA,
P0_SHA, P1_SHA and P2_SHA. Run native-service/integration/run_acceptance.sh.
It verifies those pins, the P1 import surface and slot fit, and executes
registry restart/recovery and real P2 invocation/authorization fixtures
using disposable storage and synthetic credentials.

The native-components workflow additionally builds two dedicated nested
callback probes, tests the descriptor relay, and qualifies publication
and the trusted edge. See that workflow for the complete test roster.
Historical PR53 and PR122 artifacts use an obsolete ABI and are not
valid evidence for this target.
