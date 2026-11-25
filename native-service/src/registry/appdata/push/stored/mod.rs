mod active;
mod codec;
mod prune;
mod status;
pub(super) use super::err::PushErr;
pub(super) use super::grant::{Kind, by_hash};
pub(super) use super::id;
pub(super) use super::lock::with_lock;
pub(super) use super::path::valid_key;
pub(super) use super::paths::{sub_path, subs_dir};
pub(super) use super::{Disk, MAX_SUB_BYTES};
pub(super) use crate::CAP_LEN;
pub use active::active;
pub use codec::encode;
pub(crate) use codec::parts;
pub use prune::prune;
pub use status::status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Active([u8; crate::CAP_LEN]),
    Absent,
    Mismatch,
    Invalid,
}

pub fn equal(a: &[u8], b: &[u8]) -> bool {
    let mut diff = 0u8;
    for i in 0..a.len().min(b.len()) {
        diff |= a[i] ^ b[i];
    }
    diff == 0 && a.len() == b.len()
}
