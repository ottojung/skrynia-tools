//! Disposable local HTTPS push provider: verifies VAPID and aes128gcm.
mod check;
mod decrypt;
mod decrypt_derive;
mod server;
mod tls;
mod vapid;
mod vapid_parse;
pub use check::{Verified, check};
pub use decrypt::SubKeys;
pub use server::{FakePush, ReplyPlan, Seen, read_req, write_reply};
pub use tls::{Materials, server_config};
