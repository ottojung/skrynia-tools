mod admin;
mod app;
mod app_wake;
mod dispatch;
mod operator;
mod push;
mod push_status;
mod res;
mod sender;

// Sibling op modules address these through `super::*`; re-export them here so
// the protocol op surface type-checks without nested-module imports.
pub(crate) use super::cap::{Disk, Eval, Random};
pub(crate) use crate::MutLock;
pub(crate) use crate::reg::Registry;

pub use dispatch::dispatch;
