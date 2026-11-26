use super::{Error, Interpreter, P0_IMPORT, module};

fn run(ops: &str, cb: &mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>) -> Result<Vec<u8>, Error> {
    let wat = format!("(module {P0_IMPORT} (func (export \"_start\") {ops}))");
    Interpreter::new().evaluate(&module(&wat), &[], Some(cb))
}

#[test]
fn incomplete_request_has_no_effect() {
    let mut cb = |_: &[u8]| -> Result<Vec<u8>, i32> { panic!("partial request executed") };
    assert_eq!(
        run(
            "(drop (call $invoke (i32.const 1) (i32.const 4) (i32.const 7)))",
            &mut cb
        ),
        Err(Error::Callback(-5))
    );
}

#[test]
fn multiple_complete_requests_keep_their_boundaries() {
    let ops = r#"
        (drop (call $invoke (i32.const 1) (i32.const 4) (i32.const 1)))
        (drop (call $invoke (i32.const 1) (i32.const 4) (i32.const 2)))
        (drop (call $invoke (i32.const 1) (i32.const 6) (i32.const 0)))
        (drop (call $invoke (i32.const 1) (i32.const 3) (i32.const 0)))
        (drop (call $invoke (i32.const 1) (i32.const 4) (i32.const 3)))
        (drop (call $invoke (i32.const 1) (i32.const 6) (i32.const 0)))
        (drop (call $invoke (i32.const 1) (i32.const 3) (i32.const 0)))"#;
    let mut frames = Vec::new();
    let mut cb = |req: &[u8]| {
        frames.push(req.to_vec());
        Ok(vec![99])
    };
    assert_eq!(run(ops, &mut cb), Ok(vec![]));
    assert_eq!(frames, vec![vec![1, 2], vec![3]]);
}
