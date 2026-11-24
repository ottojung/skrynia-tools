use super::abi;

mod abi_host;
mod bridge;
mod host;
mod path;
mod sys;
#[cfg(test)]
mod tests;
mod wire;

use abi_host::Abi;
pub(super) use wire::{OP_PUBLISH, OP_UNPUBLISH, dispatch};

// Local failures are negative so they can never be confused with the
// provider's positive result codes (0 OK, 1..6 provider failure).
pub const ERR_VALIDATION: i32 = -101;
pub const ERR_TICKET: i32 = -102;
pub const ERR_CLEANUP: i32 = -103;

pub fn call(op: u8, rel: &[u8]) -> i32 {
    bridge::call(&mut Abi, op == OP_PUBLISH, rel)
}
