#[cfg(target_arch = "wasm32")]
mod cb;
#[cfg(target_arch = "wasm32")]
mod entry;
#[cfg(target_arch = "wasm32")]
mod eval;
#[cfg(target_arch = "wasm32")]
pub mod lock;
#[cfg(target_arch = "wasm32")]
use super::{cap::Eval, root_abi as host};
#[cfg(target_arch = "wasm32")]
pub use lock::HostLock;

#[cfg(target_arch = "wasm32")]
pub fn set_cb_invoke(invoke_id: Vec<u8>) {
    unsafe { cb::CB_INVOKE = Some(invoke_id) };
}
#[cfg(target_arch = "wasm32")]
pub fn clear_cb_invoke() {
    unsafe { cb::CB_INVOKE = None };
}

#[cfg(target_arch = "wasm32")]
pub struct HostEval;
