//! A create-only contiguous log; writers must cooperate and not delete slots.
pub(super) use super::client::{CreateResult, Store};
mod entry;
mod read;
mod scan;
mod types;
pub use types::AppendLog;

#[cfg(test)]
mod tests;
