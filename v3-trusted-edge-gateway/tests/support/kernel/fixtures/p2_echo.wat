;; Target-ABI P1 interpreter guest: the only import is `p0.invoke` with the
;; opaque context handle 1 and the only entrypoint is `_start`. GET/PUT/FLUSH
;; echo the application's opaque input byte-for-byte; no kernel import, no
;; linear-memory pointer and no P0 callback queue is used.
(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (local $b i64)
    (block $done
      (loop $echo
        (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
        (br_if $done (i64.lt_s (local.get $b) (i64.const 0)))
        (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
        (br $echo)))
    (drop (call $invoke (i32.const 1) (i32.const 2) (i32.const 0)))))
