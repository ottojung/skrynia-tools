//! Four-byte little-endian ABI shared with the kernel HTTP syscall fallback.

use serde_json::{Value, json};

pub const OK: u32 = 0;
pub const BAD_TICKET: u32 = 1;
pub const INVALID_PATH: u32 = 2;
pub const INVALID_SOURCE: u32 = 3;
pub const DEST_CONFLICT: u32 = 4;
pub const IO_ERROR: u32 = 5;
pub const INTERNAL: u32 = 6;

/// The request body must be exactly one little-endian `u32` ticket.
pub fn decode(body: &[u8]) -> Result<u32, u32> {
    let bytes: [u8; 4] = body.try_into().map_err(|_| BAD_TICKET)?;
    Ok(u32::from_le_bytes(bytes))
}

/// A successful response is exactly four little-endian result bytes.
pub fn encode(code: u32) -> Vec<u8> {
    code.to_le_bytes().to_vec()
}

/// Advertises the two names the kernel may configure as HTTP syscalls.
pub fn syscalls_list() -> Value {
    json!(["publish", "unpublish"])
}
