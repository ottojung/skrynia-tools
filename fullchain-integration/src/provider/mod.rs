//! Disposable local HTTPS push provider: verifies VAPID and aes128gcm.
mod check;
mod decrypt;
mod server;
mod tls;
mod vapid;
pub use check::{Verified, check};
pub use decrypt::SubKeys;
pub use server::{FakePush, ReplyPlan, Seen};
