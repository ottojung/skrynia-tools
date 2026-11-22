//! Least-privilege sender surface: scoped hint list/read/ack plus a bounded
//! subscription-material fetch for delivery. No app registry, business policy
//! or raw filesystem grant.
mod list;
mod read;
mod scope;
mod sub_fetch;

pub(super) use super::Disk;
pub(super) use super::MAX_HINTS;
pub(super) use super::MAX_SUBS;
pub(super) use super::err::PushErr;
pub(super) use super::grant::{Kind, verify_grant};
pub(super) use super::hints::parse_seq;
pub(super) use super::lock::with_lock;
pub(super) use super::paths::{hint_path, hints_root, subs_dir};
pub(super) use super::stored;
pub(super) use crate::CAP_LEN;
pub(super) use crate::MutLock;
pub use list::hint_list;
pub use read::{hint_ack, hint_read};
pub(crate) use scope::scope;
pub use sub_fetch::sub_fetch;
