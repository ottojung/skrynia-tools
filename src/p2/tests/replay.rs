use super::{Error, Interpreter, module};

const GUEST: &str = r#"
(module
  (import "p0" "invoke" (func $i (param i32 i32 i32) (result i64)))
  (func (export "_start")
    (drop (call $i (i32.const 1) (i32.const 4) (i32.const 1)))
    (drop (call $i (i32.const 1) (i32.const 6) (i32.const 0)))
    (drop (call $i (i32.const 1) (i32.const 4) (i32.const 2)))
    (drop (call $i (i32.const 1) (i32.const 6) (i32.const 0)))))
"#;

#[test]
fn failed_callback_is_never_replayed() {
    let mut calls = 0;
    let mut cb = |request: &[u8]| {
        calls += 1;
        assert_eq!(request, &[1]);
        Err(-5)
    };
    let result = Interpreter::new().evaluate(&module(GUEST), &[], Some(&mut cb));
    assert_eq!(result, Err(Error::Callback(-5)));
    assert_eq!(calls, 1);
}

#[test]
fn pending_reply_cannot_be_overwritten() {
    let mut calls = 0;
    let mut cb = |_request: &[u8]| {
        calls += 1;
        Ok(vec![9, 8])
    };
    let result = Interpreter::new().evaluate(&module(GUEST), &[], Some(&mut cb));
    assert_eq!(result, Err(Error::Callback(-5)));
    assert_eq!(calls, 1);
}
