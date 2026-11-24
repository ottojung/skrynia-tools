//! Outbound WebPush request construction and injected transport.
mod encrypt;
mod https;
mod keys;
mod request;
mod topic;
mod transport;

pub use encrypt::{Rng, encrypt};
pub use https::HttpsTransport;
pub use request::PushRequest;
pub use transport::{FakeTransport, PushTransport, Reply};

pub(super) use super::error::{Error, IoFailure, Result};
pub(super) use super::sub::is_global;
pub(super) use super::sub::{Subscription, Target};
pub(super) use super::vapid::{VapidKey, authorization};
pub(super) use request::build;
pub use topic::topic_for;
