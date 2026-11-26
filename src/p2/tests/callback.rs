use super::module;
use super::{Error, Interpreter};

const GUEST: &str = r#"
(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (local $b i64)
    (local.set $b (call $invoke (i32.const 1) (i32.const 0) (i32.const 0)))
    (local.set $b (call $invoke (i32.const 1) (i32.const 3) (i32.wrap_i64 (local.get $b))))
    (if (i64.lt_s (local.get $b) (i64.const 0)) (then (return)))
    (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
    (block $done
      (loop $loop
        (local.set $b (call $invoke (i32.const 1) (i32.const 4) (i32.const 0)))
        (br_if $done (i64.eq (local.get $b) (i64.const -1)))
        (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
        (br $loop)))))
"#;

#[test]
fn appdata_callback_runs_in_p1_with_no_p0_relay() {
    let wasm = module(GUEST);
    let mut calls = 0;
    let mut cb = |request: &[u8]| {
        calls += 1;
        assert_eq!(request, &[7]);
        Ok(vec![request[0] + 1, request[0] + 2])
    };
    let out = Interpreter::new()
        .evaluate(&wasm, &[7], Some(&mut cb))
        .unwrap();
    assert_eq!(out, vec![8, 9]);
    assert_eq!(calls, 1);
}

#[test]
fn callback_op_without_capability_fails_closed() {
    let wasm = module(GUEST);
    assert_eq!(
        Interpreter::new().evaluate(&wasm, &[7], None),
        Err(Error::Callback(-4))
    );
}
