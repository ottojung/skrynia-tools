//! Full-chain acceptance scenarios against real compiled artifacts.
pub(crate) mod keys;
mod negative;
mod p1;
mod positive;
pub(crate) mod transport;
mod wire;
mod wake;

pub use keys::fixture;
pub use negative::run as run_negative;
pub use p1::{call, must};
pub use positive::run_positive;
pub use transport::LoopbackPush;
pub use wire::{blb, idb, req, req_ver, u64b};

pub use wake::run as run_wake;
