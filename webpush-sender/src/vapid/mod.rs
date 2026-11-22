//! ES256 VAPID JWT. The private key never leaves this module.
mod header;
mod sign;

pub use header::authorization;
pub use sign::VapidKey;

pub(super) use super::error::{Error, Result};
