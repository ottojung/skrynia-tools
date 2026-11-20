#[cfg(target_arch = "wasm32")]
mod disk;
#[cfg(target_arch = "wasm32")]
mod eval;
#[cfg(target_arch = "wasm32")]
mod lock;
#[cfg(target_arch = "wasm32")]
mod random;

#[cfg(target_arch = "wasm32")]
pub use disk::CbDisk;
#[cfg(target_arch = "wasm32")]
pub use eval::CbEval;
#[cfg(target_arch = "wasm32")]
pub use lock::CbLock;
#[cfg(target_arch = "wasm32")]
pub use random::CbRandom;
