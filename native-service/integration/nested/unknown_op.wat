(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (local $code i64)
    ;; An unrecognised p0.invoke op must fail closed with EINVAL (-3),
    ;; never be silently ignored or treated as a successful no-op.
    (local.set $code (call $invoke (i32.const 1) (i32.const 99) (i32.const 0)))
    ;; Emit the low byte of the returned code so the harness can assert it.
    (drop (call $invoke (i32.const 1) (i32.const 1)
      (i32.and (i32.wrap_i64 (local.get $code)) (i32.const 255))))))
