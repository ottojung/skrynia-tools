pub mod root_abi;
pub mod root_disk;
pub mod root_disk_atom;
pub mod root_disk_io;
pub mod root_disk_ls;
pub mod root_eval;
pub mod root_main;
pub mod root_random;
#[cfg(target_arch = "wasm32")]
pub(crate) use super::cap;
#[cfg(target_arch = "wasm32")]
pub(crate) use super::{handle, id, reg};
pub mod root_stream;
