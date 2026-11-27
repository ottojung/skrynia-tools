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
        (br_if $done (i64.lt_s (local.get $b) (i64.const 0)))
        (drop (call $invoke (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
        (br $loop)))))
"#;

#[test]
fn response_cap_bounds_host_allocation() {
    let wasm = module(ECHO);
    let evaluator = Interpreter::with(Binding::p0(), Limits::output(2));
    assert_eq!(
        evaluator.evaluate(&wasm, &[1, 2, 3, 4], None),
        Err(Error::Callback(-6))
    );
}
