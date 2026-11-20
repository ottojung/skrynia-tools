#[cfg(target_arch = "wasm32")]
pub(super) use super::root_abi;
pub mod input;
pub mod output;
#[cfg(target_arch = "wasm32")]
pub use input::read_all;
#[cfg(target_arch = "wasm32")]
pub use output::write_all;
