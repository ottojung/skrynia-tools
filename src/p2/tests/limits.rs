use super::module;
use super::{Binding, Interpreter, Limits};

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
fn response_cap_bounds_host_allocation() {
    let wasm = module(ECHO);
    let evaluator = Interpreter::with(Binding::p0(), Limits::output(2));
    let out = evaluator.evaluate(&wasm, &[1, 2, 3, 4], None).unwrap();
    assert_eq!(out, vec![1, 2]);
}
