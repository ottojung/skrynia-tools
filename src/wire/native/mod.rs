//! The ordinary native application service's binary POST protocol.
mod caps;
mod request;
mod response;
pub use caps::{Capabilities, digest, parse_cap};
pub use request::{delete, get, put};
pub use response::{deleted, read, written};
pub const EDGE_HEADER: &str = "X-Skrynia-Edge-Capability";
pub const MAX_FRAME: usize = 16 * 1024 * 1024;
use super::valid_key;
#[cfg(test)]
mod tests;
