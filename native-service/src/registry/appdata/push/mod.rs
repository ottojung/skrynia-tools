mod err;
mod grant;
mod hints;
mod lock;
mod paths;
mod sender;
mod stored;
mod subscribe;
mod unsubscribe;

pub(crate) use super::cap::Disk;
pub(crate) use super::id;
pub(crate) use super::path;
pub use err::PushErr;
pub use grant::{Kind, issue as issue_grant, verify as verify_grant};
pub use hints::{enqueue_hints, mark_wake_dirty};
pub(crate) use hints::{wake_ack, wake_current, wake_read};
pub use lock::with_lock;
pub use paths::*;
pub use sender::{hint_ack, hint_list, hint_read, sub_fetch};
pub use subscribe::subscribe;
pub use unsubscribe::unsubscribe;

/// Reserved exact key for generic namespace wake fan-out. A subscriber that
/// holds an ordinary exact-key Subscribe grant for this key receives an
/// invalidation hint for every mutation in its namespace. No prefix matching,
/// no application semantics: senders still re-read authoritative state.
pub const WAKE_KEY: &[u8] = b"skrynia-wake-v1";

pub const MAX_SUB_BYTES: usize = 4096;
pub const MAX_SUBS: usize = 64;
pub const MAX_HINTS: usize = 1024;
