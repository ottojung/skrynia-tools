(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (local $b i64)
    ;; GET the single opaque selector byte from P1 input.
    (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
    ;; CB_GET: P1's own appdata callback executes on P1's stack.
    (local.set $b (call $invoke (i32.const 1) (i32.const 3) (i32.wrap_i64 (local.get $b))))
    (if (i64.lt_s (local.get $b) (i64.const 0)) (then unreachable))
    (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
    ;; CB_PUT: drain the remaining reply bytes into the response stream.
    (block $done
      (loop $loop
        (local.set $b (call $invoke (i32.const 1) (i32.const 4) (i32.const 0)))
        (br_if $done (i64.eq (local.get $b) (i64.const -1)))
        (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
        (br $loop)))))
