(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (local $a i64)
    (local $b i64)
    ;; Unsupported selector -> P1 callback returns an error; failure is sticky.
    (local.set $a (call $invoke (i32.const 1) (i32.const 3) (i32.const 238)))
    ;; A second CB_GET must return the same code without re-entering P1's callback.
    (local.set $b (call $invoke (i32.const 1) (i32.const 3) (i32.const 0)))
    (if (i64.ne (local.get $a) (local.get $b)) (then unreachable))
    ;; STATUS also reports the sticky failure, never a fresh success.
    (drop (call $invoke (i32.const 1) (i32.const 5) (i32.const 0)))))
