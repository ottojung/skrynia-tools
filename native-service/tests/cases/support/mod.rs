mod disk;
mod fsdisk;
mod lock;
mod random;
pub use disk::MemDisk;
pub use fsdisk::FsDisk;
pub use lock::{NoLock, StdLock};
pub use random::Rng;
