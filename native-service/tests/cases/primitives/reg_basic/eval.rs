use skrynia_native_service::cap::Eval;

pub struct EchoEval;
impl Eval for EchoEval {
    type Err = ();
    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok([wasm, input].concat())
    }
}
