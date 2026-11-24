pub mod auth;
pub mod cap;
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub mod cb_guest;
#[cfg(not(all(target_arch = "wasm32", feature = "cb-guest")))]
pub mod host;
pub mod mlock;
pub mod protocol;
pub mod registry;

pub use auth::{CAP_LEN, hash_bytes, read_cap, read_cap_vec};
#[cfg(not(all(target_arch = "wasm32", feature = "cb-guest")))]
pub use host::{
    root_abi, root_disk, root_disk_atom, root_disk_io, root_disk_ls, root_eval, root_random,
};
pub use mlock::{MutLock, NoLock};
pub use protocol::{handle, wdec, wenc, wire};
pub use registry::appdata::{AppErr, PushErr, WriteAuth};
pub use registry::{id, rcache, reg, reg_ops, store, store_io, store_ls};
