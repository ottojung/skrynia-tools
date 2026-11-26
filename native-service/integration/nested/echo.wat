(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (local $b i64)
    (block $done
      (loop $loop
        (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
        (br_if $done (i64.eq (local.get $b) (i64.const -1)))
        (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
        (br $loop)))))
