mod path;
use crate::{hash_bytes, read_cap};

pub use path::{active_path, blob_path, ready_dir, ready_path};

mod active;
pub use active::{ActiveState, decode_active, encode_active};
pub const BLOB_VER: u8 = 1;

pub fn encode_blob(code: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + code.len());
    out.push(BLOB_VER);
    out.extend_from_slice(code);
    out
}

pub fn decode_blob(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.first().copied()? != BLOB_VER {
        return None;
    }
    Some(bytes[1..].to_vec())
}

pub fn code_hash(code: &[u8]) -> [u8; 32] {
    hash_bytes(code)
}

pub fn invoke_id(admin_cap: &[u8]) -> [u8; 32] {
    read_cap(admin_cap)
}
