use super::module;
use super::{Error, Interpreter};

const GUEST: &str = r#"
(module
  (import "p0" "invoke" (func $invoke (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (drop (call $invoke (i32.const 1) (i32.const 3) (i32.const 1)))
    (drop (call $invoke (i32.const 1) (i32.const 3) (i32.const 2)))))
"#;

#[test]
fn failed_callback_is_never_replayed() {
    let wasm = module(GUEST);
    let mut calls = 0;
    let mut cb = |_request: &[u8]| {
        calls += 1;
        Err(-5)
    };
    let result = Interpreter::new().evaluate(&wasm, &[], Some(&mut cb));
    assert_eq!(result, Err(Error::Callback(-5)));
    assert_eq!(calls, 1, "sticky failure must prevent a second P1 callback");
}
