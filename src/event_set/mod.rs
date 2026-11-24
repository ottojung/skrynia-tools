//! Optional grow-only events, assuming create-only authority and cooperating writers.
//! Listing is not a snapshot: inserts before a pagination cursor can be missed.
pub(super) use super::client::{CreateResult, Store};
pub(super) use super::wire;
mod ids;
mod insert;
mod page;
mod scan;
mod types;
pub use types::{Event, EventPage, EventScan, EventSet, EventStore, Insert};
#[cfg(test)]
mod tests;
