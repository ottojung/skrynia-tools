//! Parse sender-scoped subscription material and validate endpoints.
mod endpoint;
mod host;
mod ip;
mod record;
mod scope;
mod subscription;

pub use endpoint::{Resolver, Target, validate_url};
pub use record::{parse_frame, parse_frame_isolated};
pub use scope::check_scope;
pub use subscription::Subscription;

pub(crate) use ip::is_global;

pub(super) use super::config::{CAP_LEN, MAX_KEY, MAX_SUBS};
pub(super) use super::error::{Error, IoFailure, Result};
