#[cfg(target_arch = "wasm32")]
mod map;
#[cfg(target_arch = "wasm32")]
mod ops;
#[cfg(target_arch = "wasm32")]
mod run;

#[cfg(target_arch = "wasm32")]
pub(super) static mut CB_INVOKE: Option<Vec<u8>> = None;

#[cfg(target_arch = "wasm32")]
pub fn dispatch(invoke_id: &[u8], req: &[u8]) -> Result<Vec<u8>, i32> {
    run::dispatch(invoke_id, req)
}
