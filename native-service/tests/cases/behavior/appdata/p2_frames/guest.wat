(module
(import "p0" "invoke" (func $i (param i32 i32 i32) (result i64)))
(func (export "_start") (local $b i64)
  (block $end (loop $read
    (local.set $b (call $i (i32.const 1) (i32.const 0) (i32.const 0)))
    (br_if $end (i64.lt_s (local.get $b) (i64.const 0)))
    (drop (call $i (i32.const 1) (i32.const 4) (i32.wrap_i64 (local.get $b))))
    (br $read)))
  (drop (call $i (i32.const 1) (i32.const 6) (i32.const 0)))
  (block $end (loop $read
    (local.set $b (call $i (i32.const 1) (i32.const 3) (i32.const 0)))
    (br_if $end (i64.lt_s (local.get $b) (i64.const 0)))
    (drop (call $i (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
    (br $read)))))
