//! Fixed-slot durable dirty generation for the reserved wake key.
//! One atomic state per exact (namespace, wake_key); no per-generation files,
//! no pruning, O(1) storage under an offline sender or a remove-denied Disk.
mod format;
mod ops;
mod sender;

pub(super) use super::{Disk, PushErr, wake_slot_path};
pub(crate) use format::{decode, encode};
pub use ops::{current, mark_dirty};
pub use sender::{ack, read};
