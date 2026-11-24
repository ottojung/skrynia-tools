//! Constructors for the full-root capacity fixture.
use super::capacity_root::key;
use sha2::{Digest, Sha256};

pub fn seal(mut b: Vec<u8>) -> Vec<u8> {
    b.extend_from_slice(&Sha256::digest(&b));
    b
}
pub fn set_dir(b: &mut [u8], i: usize, k: [u8; 32], id: u32) {
    b[i * 36..i * 36 + 32].copy_from_slice(&k);
    b[i * 36 + 32..i * 36 + 36].copy_from_slice(&id.to_le_bytes());
}
pub fn leaf(start: u32, n: u32) -> Vec<u8> {
    let mut b = vec![0u8; 124 * 33];
    for i in 0..n as usize {
        b[i * 33..i * 33 + 32].copy_from_slice(&key(start + i as u32));
        b[i * 33 + 32] = 1;
    }
    seal(b)
}
