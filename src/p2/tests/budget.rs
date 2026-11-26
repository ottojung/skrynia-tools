use super::{Binding, Error, Interpreter, Limits, P0_IMPORT, module};

#[test]
fn userland_fuel_stops_infinite_guest_and_next_invocation_recovers() {
    let loop_wat = format!("(module {P0_IMPORT} (func (export \"_start\") (loop $l (br $l))))");
    let evaluator = Interpreter::with(Binding::p0(), Limits::default().with_fuel(1000));
    assert_eq!(
        evaluator.evaluate(&module(&loop_wat), &[], None),
        Err(Error::Budget)
    );
    let empty = format!("(module {P0_IMPORT} (func (export \"_start\")))");
    assert_eq!(evaluator.evaluate(&module(&empty), &[], None), Ok(vec![]));
}

#[test]
fn userland_linear_memory_limit_rejects_oversized_initial_memory() {
    let large = format!("(module {P0_IMPORT} (memory 1025) (func (export \"_start\")))");
    assert_eq!(
        Interpreter::new().evaluate(&module(&large), &[], None),
        Err(Error::Trap)
    );
}

#[test]
fn oversized_callback_frame_never_executes() {
    let src = format!(
        "(module {P0_IMPORT} (func (export \"_start\")
        (drop (call $invoke (i32.const 1) (i32.const 4) (i32.const 0)))
        (drop (call $invoke (i32.const 1) (i32.const 4) (i32.const 1)))
        (drop (call $invoke (i32.const 1) (i32.const 6) (i32.const 0)))))"
    );
    let evaluator = Interpreter::with(Binding::p0(), Limits::default().with_callback_cap(1));
    let mut cb = |_: &[u8]| -> Result<Vec<u8>, i32> { panic!("oversized request executed") };
    assert_eq!(
        evaluator.evaluate(&module(&src), &[], Some(&mut cb)),
        Err(Error::Callback(-6))
    );
}
