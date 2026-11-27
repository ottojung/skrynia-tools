use super::{Error, Interpreter, module};

const GUEST: &str = r#"
(module
  (import "p0" "invoke" (func $i (param i32 i32 i32) (result i64)))
  (func (export "_start") (local $b i64)
    (block $done (loop $loop
      (local.set $b (call $i (i32.const 1) (i32.const 0) (i32.const 0)))
      (br_if $done (i64.eq (local.get $b) (i64.const -1)))
      (if (i64.ne (call $i (i32.const 1) (i32.const 4) (i32.wrap_i64 (local.get $b))) (i64.const 0))
        (then (return)))
      (br $loop)))
    (if (i64.ne (call $i (i32.const 1) (i32.const 6) (i32.const 0)) (i64.const 0))
      (then (return)))
    (block $done (loop $loop
      (local.set $b (call $i (i32.const 1) (i32.const 3) (i32.const 0)))
      (br_if $done (i64.eq (local.get $b) (i64.const -1)))
      (drop (call $i (i32.const 1) (i32.const 1) (i32.wrap_i64 (local.get $b))))
      (br $loop)))))
"#;

#[test]
fn opaque_multibyte_callback_roundtrips_exactly() {
    let mut calls = 0;
    let input: Vec<u8> = (0..=255).cycle().take(8193).collect();
    let mut cb = |request: &[u8]| {
        calls += 1;
        assert_eq!(request, input);
        Ok(request.iter().rev().copied().collect())
    };
    let out = Interpreter::new()
        .evaluate(&module(GUEST), &input, Some(&mut cb))
        .unwrap();
    assert_eq!(out, input.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(calls, 1);
}

#[test]
fn callback_without_capability_fails_closed() {
    assert_eq!(
        Interpreter::new().evaluate(&module(GUEST), &[7], None),
        Err(Error::Callback(-4))
    );
}
