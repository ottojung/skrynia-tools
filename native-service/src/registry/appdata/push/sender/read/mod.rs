//! Exact-scope sender reads and conditional acknowledgement.
mod ack;
mod get;
pub(super) use super::{
    CAP_LEN, DATA_SLOT_FLAG, Disk, MutLock, PushErr, WAKE_KEY, hint_path, scope, wake_ack,
    wake_read, with_lock,
};
pub use ack::hint_ack;
pub use get::hint_read;
