pub mod auth;
pub mod cap;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub mod guest_sys;
#[cfg(not(all(target_arch = "wasm32", feature = "cb-guest")))]
pub mod host;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod invoke_ctx;
pub mod mlock;
pub mod protocol;
pub mod registry;

#[cfg(not(all(target_arch = "wasm32", feature = "cb-guest")))]
pub use host::{
    root_abi, root_disk, root_disk_atom, root_disk_io, root_disk_ls, root_eval, root_random,
};
pub use protocol::{handle, wdec, wenc, wire};
pub use registry::{id, rcache, reg, reg_ops, store, store_io, store_ls};

pub use auth::{CAP_LEN, hash_bytes, read_cap, read_cap_vec};
pub use mlock::MutLock;
pub use registry::appdata::{AppErr, WriteAuth};

#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub fn set_invoke_ctx(id: Vec<u8>) {
    invoke_ctx::set(id);
}
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub fn clear_invoke_ctx() {
    invoke_ctx::clear();
}
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub fn take_invoke_ctx() -> Option<Vec<u8>> {
    invoke_ctx::get()
}
