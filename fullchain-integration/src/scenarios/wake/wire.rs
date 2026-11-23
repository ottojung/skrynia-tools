//! Exact P1 on-wire request fixtures for the compiled ordinary WASM.
use crate::{blb, idb, must, req, u64b};
use super::ctx::Ctx;
use sha2::{Digest, Sha256};
pub const KEY: &[u8] = b"skrynia-wake-v1";
pub fn h(b: &[u8]) -> [u8; 32] { Sha256::digest(b).into() }
pub fn owner() -> [u8; 32] { h(b"wake-owner") }
pub fn a() -> [u8; 32] { h(b"wake-source-a") }
pub fn b() -> [u8; 32] { h(b"wake-source-b") }
pub fn grant_a() -> [u8; 32] { h(b"wake-grant-a") }
pub fn grant_b() -> [u8; 32] { h(b"wake-grant-b") }
pub fn sender() -> [u8; 32] { h(b"wake-sender") }
pub fn ns() -> [u8; 32] { h(&owner()) }
pub fn write(c: &Ctx, cap: &[u8], grant: &[u8], key: &[u8], value: &[u8]) {
    must(&c.io, &req(25, &[
        &idb(cap), &blb(key), &[0], &blb(value), &idb(grant), &blb(KEY),
    ]), "wake-put");
}
pub fn ack(c: &Ctx, n: u64) {
    must(&c.io, &req(22, &[
        &idb(&sender()), &idb(&ns()), &blb(KEY), &u64b(n),
    ]), "wake-ack");
}
pub fn list(c: &Ctx) -> Vec<u64> {
    let v = must(&c.io, &req(20, &[
        &idb(&sender()), &idb(&ns()), &blb(KEY),
    ]), "wake-list");
    let n = u32::from_le_bytes(v[..4].try_into().unwrap()) as usize;
    assert_eq!(v.len(), 4 + 8 * n);
    (0..n).map(|i| u64::from_le_bytes(v[4+i*8..12+i*8].try_into().unwrap())).collect()
}
