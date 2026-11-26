//! Outbound WebPush request construction and injected transport.
mod encrypt;
mod https;
mod keys;
pub mod proc;
mod request;
mod topic;
mod transport;

pub use encrypt::{Rng, encrypt};
pub use https::HttpsTransport;
pub use request::PushRequest;
pub use transport::{FakeTransport, PushTransport, Reply};

pub(super) use proc::{fatal, keep_worse};

pub(super) use super::config::Config;
pub(super) use super::error::{Error, IoFailure, Result};
pub(super) use super::ledger::FileLedgerStore;
pub(super) use super::p1::{Edge, HttpP1, P1Transport};
pub(super) use super::pump::{Clock, Parts, Pump};
pub(super) use super::sub::{Resolver, Subscription, Target, is_global};
pub(super) use super::vapid::{VapidKey, authorization};
pub(super) use request::build;
pub use topic::topic_for;
