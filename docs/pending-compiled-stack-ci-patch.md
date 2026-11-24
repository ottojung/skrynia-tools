# Pending compiled-stack CI update (workflow-scope credential required)

This agent's GitHub OAuth token lacks the `workflow` scope, so GitHub
refused a push that modifies `.github/workflows/native-components.yml`.
The exact patch below must be applied by a workflow-scoped actor to
complete the kernel pin and compiled regression gates. Saved at
/tmp/opencode/pr98fix/evidence/workflow-kernel-p2.patch on the host.

```diff
diff --git a/.github/workflows/native-components.yml b/.github/workflows/native-components.yml
index 8bda132..09ac0ce 100644
--- a/.github/workflows/native-components.yml
+++ b/.github/workflows/native-components.yml
@@ -29,7 +29,7 @@ jobs:
       - uses: actions/checkout@v5
         with:
           repository: ottojung/Skrynia
-          ref: dc1170c9f426c45bd9d098fecbf746d9db7176a2
+          ref: 6e750f6a737a744495f98a84830b53b44186f38f
           path: upstream-skrynia
           persist-credentials: false
       - uses: dtolnay/rust-toolchain@1.93.0
@@ -43,11 +43,31 @@ jobs:
             v3-trusted-edge-gateway -> target
       - name: Build generic kernel
         working-directory: upstream-skrynia
-        run: cargo build --locked --release -p skrynia-server
+        run: cargo build --locked --release -p libskrynia --bin skrynia-server
+      - name: Kernel nested-callback LEAK->SAFE regression (compiled)
+        working-directory: upstream-skrynia
+        run: cargo test --locked -p libskrynia --test http callback::nested::nested_guest_cannot_steal_root_disk_fd_after_callback -- --exact
       - name: Build ordinary guests
         run: |
           cargo +1.93.0 build --locked --release --target wasm32-unknown-unknown -p skrynia-program0
           cargo +1.93.0 build --locked --release --target wasm32-unknown-unknown -p skrynia-native-service --features cb-guest
+      - name: Build current Theorem P2 guest
+        env:
+          CARGO_TARGET_DIR: ${{ github.workspace }}/target
+        run: cargo +1.93.0 build --locked --release --target wasm32-unknown-unknown --manifest-path examples/theorem_lab_feedback_p2/Cargo.toml
+      - name: Real compiled-stack P2 adversarial fixture
+        run: |
+          set -eu
+          KERNEL="$PWD/upstream-skrynia/target/release/skrynia-server"
+          P0="$PWD/target/wasm32-unknown-unknown/release/skrynia_program0.wasm"
+          P1="$PWD/target/wasm32-unknown-unknown/release/skrynia_native_service.wasm"
+          P2="$PWD/target/wasm32-unknown-unknown/release/theorem_lab_feedback_p2.wasm"
+          python3 examples/theorem_lab_feedback_p2/tests/real_kit_p2.py \
+            --kernel "$KERNEL" --p0 "$P0" --p1 "$P1" --p2 "$P2" \
+            --kernel-sha256 "$(sha256sum "$KERNEL" | cut -d' ' -f1)" \
+            --p0-sha256 "$(sha256sum "$P0" | cut -d' ' -f1)" \
+            --p1-sha256 "$(sha256sum "$P1" | cut -d' ' -f1)" \
+            --p2-sha256 "$(sha256sum "$P2" | cut -d' ' -f1)"
       - name: Compiled service and trusted-edge tests
         run: |
           export SKRYNIA_REQUIRE_KIT=1
```
