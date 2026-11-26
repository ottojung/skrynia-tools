use super::{Error, Interpreter};
use super::{P0_IMPORT, module};

#[test]
fn guest_trap_is_a_typed_error_not_a_partial_result() {
    let src = format!("(module {P0_IMPORT} (func (export \"_start\") unreachable))");
    let wasm = module(&src);
    assert_eq!(
        Interpreter::new().evaluate(&wasm, &[], None),
        Err(Error::Trap)
    );
}
