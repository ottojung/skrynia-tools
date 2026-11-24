//! Shared P1 appdata helpers for the wake regression integration tests.
#![allow(dead_code)]
mod app;
pub mod crosskey;
pub mod crosskey_query;
mod faildata;
mod fault;
mod memdisk;
mod queries;
mod sender;
mod wake;

#[allow(unused_imports)] // Shared between independent integration-test roots.
pub use app::{caps, put, subscribe, unsubscribe};
#[allow(unused_imports)] // Shared between independent integration-test roots.
pub use faildata::FailData;
#[allow(unused_imports)]
pub use fault::{NoRemove, NoSlotWrite};
pub use memdisk::Mem;
#[allow(unused_imports)]
pub use queries::{data_exists, hints};
#[allow(unused_imports)] // Shared between independent integration-test roots.
pub use sender::{SENDER_A, SENDER_B, ns, seed_sender, seeded};
#[allow(unused_imports)]
pub use wake::{ack, list, put_ok, read_ok};

pub const WAKE: &[u8] = b"skrynia-wake-v1";
