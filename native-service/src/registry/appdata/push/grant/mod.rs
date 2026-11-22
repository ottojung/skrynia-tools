//! Explicit narrow grants. A subscribe grant authorizes exactly one exact
//! object key for one subscriber; a sender scope authorizes hint access to
//! exactly one (namespace, key) pair. Only a namespace writer may issue them.
mod fmt;
mod issue;
mod record;
mod snapshot;
mod verify;

pub(super) use super::Disk;
pub(super) use super::err;
pub(super) use super::id;
pub(super) use super::lock::with_lock;
pub(super) use super::path;
pub use fmt::Kind;
pub use issue::issue;
pub use verify::verify;
pub use verify::verify as verify_grant;

pub(super) use record::{decode, encode, owned_by, revision};
pub(super) use snapshot::by_hash;
