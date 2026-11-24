//! On-wire P1 exact-key request helpers with the full u64 sender sequence.
use super::ctx::Ctx;
use crate::{blb, idb, must, req, u64b};
use sha2::{Digest, Sha256};
pub const KEY: &[u8] = b"ordinary-exact-data-key-v1";
pub fn h(b: &[u8]) -> [u8; 32] { Sha256::digest(b).into() }
pub fn writer() -> [u8; 32] { h(b"ordinary-writer") }
pub fn ns() -> [u8; 32] { h(&writer()) }
pub fn sender() -> [u8; 32] { h(b"ordinary-sender-grant") }
pub fn list(c: &Ctx) -> Vec<u64> {
    let b = must(&c.io, &req(20, &[
        &idb(&sender()), &idb(&ns()), &blb(KEY),
    ]), "ordinary list");
    let n = u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
    assert_eq!(b.len(), 4 + n*8);
    (0..n).map(|i| u64::from_le_bytes(b[4+i*8..12+i*8].try_into().unwrap())).collect()
}
pub fn read(c: &Ctx, seq: u64) -> Vec<u8> {
    must(&c.io, &req(21, &[
        &idb(&sender()), &idb(&ns()), &blb(KEY), &u64b(seq),
    ]), "ordinary read")
}
