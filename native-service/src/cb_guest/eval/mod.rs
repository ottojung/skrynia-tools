#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod cb;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod kernel;

#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub use cb::{clear_invoke, set_invoke};

#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub struct MediatorEval;

#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
impl crate::cap::Eval for MediatorEval {
    type Err = i32;

    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        kernel::eval_nested(wasm, input)
    }
}
