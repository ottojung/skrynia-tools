//! Target typed integer wire ABI: version 1, one i32 value.

use serde_json::{Value, json};

pub const OK: u32 = 0;
pub const BAD_TICKET: u32 = 1;
pub const INVALID_PATH: u32 = 2;
pub const INVALID_SOURCE: u32 = 3;
pub const DEST_CONFLICT: u32 = 4;
pub const IO_ERROR: u32 = 5;
pub const INTERNAL: u32 = 6;

/// One i32 ticket encoded as version, count, tag, and four LE bytes.
pub fn decode(body: &[u8]) -> Result<u32, u32> {
    let [1, 1, 0x32, a, b, c, d] = body else {
        return Err(BAD_TICKET);
    };
    let bytes = [*a, *b, *c, *d];
    Ok(u32::from_le_bytes(bytes))
}

/// The result uses the same typed i32 wire representation.
pub fn encode(code: u32) -> Vec<u8> {
    let mut result = vec![1, 1, 0x32];
    result.extend_from_slice(&code.to_le_bytes());
    result
}

/// Advertises the complete list of supported syscalls.
pub fn syscalls_list() -> Value {
    json!({"functions": [
        {"name": "publish", "params": ["i32"], "results": ["i32"]},
        {"name": "unpublish", "params": ["i32"], "results": ["i32"]}
    ]})
}
