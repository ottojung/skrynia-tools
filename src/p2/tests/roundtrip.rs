use super::module;
use super::{Binding, Error, Interpreter, Limits};

const ECHO: &str = r#"
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
"#;

#[test]
fn echo_guest_roundtrips_input_bytes() {
    let wasm = module(ECHO);
    let out = Interpreter::new()
        .evaluate(&wasm, &[1, 2, 255], None)
        .unwrap();
    assert_eq!(out, vec![1, 2, 255]);
}

const STATUS_GUEST: &str = r#"
(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (if (i64.ne (call $invoke (i32.const 1) (i32.const 5) (i32.const 0)) (i64.const 0))
      (then unreachable))))
"#;

#[test]
fn wrong_context_handle_is_rejected() {
    let wasm = module(STATUS_GUEST);
    let interpreter = Interpreter::with(Binding::p0().with_context(9), Limits::default());
    assert_eq!(interpreter.evaluate(&wasm, &[], None), Err(Error::Trap));
    assert!(Interpreter::new().evaluate(&wasm, &[], None).is_ok());
}
