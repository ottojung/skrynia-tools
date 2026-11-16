//! A create-only contiguous log; writers must cooperate and not delete slots.
pub(super) use super::client::{CreateResult, Store};
pub(super) use super::wire as entry;
pub(super) use super::wire;
mod race;
mod read;
mod scan;
mod types;
pub use types::AppendLog;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod concurrent;

#[cfg(test)]
mod race_test;
