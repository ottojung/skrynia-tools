#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod call;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod disk;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod disk_atom;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod disk_ls;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod disk_read;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod disk_rm;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod disk_write;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod lock;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
mod wire;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub use disk::CbDisk;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub use lock::{CbLock, CbRandom};
