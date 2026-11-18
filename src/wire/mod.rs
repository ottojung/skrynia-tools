//! Portable, transport-neutral Skrynia protocol byte rules.
mod entry;
mod observed;
mod slot;
pub mod v2compat;
pub use entry::{decode, encode};
pub use observed::observed;
pub use slot::{key, valid_key, valid_matcher, valid_prefix};
