//! P1 evaluates P2 and forwards complete opaque request frames to its callback.
pub struct P1CallbackEval;

fn code(err: crate::p2::Error) -> i32 {
    match err {
        crate::p2::Error::Callback(c) => c,
        crate::p2::Error::NoCallback => -4,
        crate::p2::Error::Trap => -2,
        crate::p2::Error::InvalidWasm => -1,
        crate::p2::Error::Budget => -6,
        _ => -3,
    }
}

impl crate::cap::Eval for P1CallbackEval {
    type Err = i32;

    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        crate::p2::Interpreter::new()
            .evaluate(wasm, input, cb)
            .map_err(code)
    }
}
