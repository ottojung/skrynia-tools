//! Fixed-width, checksummed, immutable index node encodings.
use sha2::{Digest, Sha256};

pub const MAGIC: &[u8; 4] = b"IX01";
pub const META_LEN: usize = 48;

fn sum(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

pub fn seal(mut body: Vec<u8>) -> Vec<u8> {
    let c = sum(&body);
    body.extend_from_slice(&c);
    body
}

pub fn open(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 32 {
        return None;
    }
    let split = bytes.len() - 32;
    if sum(&bytes[..split]) != bytes[split..] {
        return None;
    }
    Some(bytes[..split].to_vec())
}

pub fn checked(bytes: &[u8], len: usize) -> Option<Vec<u8>> {
    if bytes.len() != len + 32 {
        return None;
    }
    open(bytes)
}

pub fn parse_meta(body: &[u8]) -> Option<(u32, u32, u32)> {
    if body.len() != META_LEN - 32 || &body[..4] != MAGIC {
        return None;
    }
    Some((
        u32::from_le_bytes(body[4..8].try_into().ok()?),
        u32::from_le_bytes(body[8..12].try_into().ok()?),
        u32::from_le_bytes(body[12..16].try_into().ok()?),
    ))
}

pub fn make_meta(dirs: u32, leaves: u32, root: u32) -> Vec<u8> {
    let mut b = Vec::with_capacity(META_LEN);
    b.extend_from_slice(MAGIC);
    b.extend_from_slice(&dirs.to_le_bytes());
    b.extend_from_slice(&leaves.to_le_bytes());
    b.extend_from_slice(&root.to_le_bytes());
    seal(b)
}
