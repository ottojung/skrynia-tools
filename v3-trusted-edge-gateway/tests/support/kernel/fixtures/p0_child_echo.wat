;; Target-ABI child of the compiled dumb P0: the only import is `p0.invoke`
;; and the only entrypoint is `_start`. P0 accepts relay writes only as framed
;; WRITE requests, so the fixture sends `op=3, fd=1, len, bytes`, verifies
;; PUT/FLUSH status, reconstructs the signed i32 write count, requires it to
;; equal n, then drains reply EOF. Input EOF (-1) is distinguished from errors.
(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (memory (export "memory") 4)
  (func (export "_start")
    (local $b i64) (local $n i32) (local $i i32) (local $eof i32)
    (local $rc i64) (local $count i32)
    (block $exit
      (loop $chunks
        (local.set $n (i32.const 0))
        (local.set $eof (i32.const 0))
        (block $read_done
          (loop $read
            (br_if $read_done (i32.ge_u (local.get $n) (i32.const 32768)))
            (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
            (if (i64.eq (local.get $b) (i64.const -1))
              (then (local.set $eof (i32.const 1)) (br $read_done)))
            (if (i64.lt_s (local.get $b) (i64.const 0)) (then unreachable))
            (i32.store8 (i32.add (i32.const 65536) (local.get $n))
              (i32.wrap_i64 (local.get $b)))
            (local.set $n (i32.add (local.get $n) (i32.const 1)))
            (br $read)))
        (if (i32.and (i32.eqz (local.get $n)) (local.get $eof))
          (then (br $exit)))
        (i32.store8 (i32.const 0) (i32.const 3))
        (i32.store (i32.const 1) (i32.const 1))
        (i32.store (i32.const 5) (local.get $n))
        (local.set $i (i32.const 0))
        (block $copy_done
          (loop $copy
            (br_if $copy_done (i32.ge_u (local.get $i) (local.get $n)))
            (i32.store8 (i32.add (i32.const 9) (local.get $i))
              (i32.load8_u (i32.add (i32.const 65536) (local.get $i))))
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $copy)))
        (local.set $i (i32.const 0))
        (block $put_done
          (loop $put
            (br_if $put_done (i32.ge_u (local.get $i) (i32.add (i32.const 9) (local.get $n))))
            (local.set $rc (call $invoke (i32.const 1) (i32.const 1)
              (i32.load8_u (local.get $i))))
            (if (i64.ne (local.get $rc) (i64.const 0)) (then unreachable))
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $put)))
        (local.set $rc (call $invoke (i32.const 1) (i32.const 2) (i32.const 0)))
        (if (i64.ne (local.get $rc) (i64.const 0)) (then unreachable))
        (local.set $count (i32.const 0))
        (local.set $i (i32.const 0))
        (block $resp_done
          (loop $resp
            (br_if $resp_done (i32.ge_u (local.get $i) (i32.const 4)))
            (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
            (if (i64.lt_s (local.get $b) (i64.const 0)) (then unreachable))
            (if (i64.gt_s (local.get $b) (i64.const 255)) (then unreachable))
            (local.set $count (i32.or (local.get $count)
              (i32.shl (i32.wrap_i64 (local.get $b))
                (i32.shl (local.get $i) (i32.const 3)))))
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $resp)))
        (if (i32.ne (local.get $count) (local.get $n)) (then unreachable))
        (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
        (if (i64.ne (local.get $b) (i64.const -1)) (then unreachable))
        (br_if $exit (local.get $eof))
        (br $chunks)))))
