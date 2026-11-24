//! Shared exact-key subscription and sender-visible wake list helpers.
use crate::common::Mem;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::path::data_path;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, hash_bytes, read_cap};

pub const KEY: &[u8] = b"data-key-v1";
pub const SUB: [u8; 32] = [5u8; 32];
pub const SENDER: [u8; 32] = [8u8; 32];
pub const WAKE_OWNER: [u8; 32] = [1u8; 32];

/// Exact-key Subscribe grant and subscription for `owner` on `key`.
pub fn sub(d: &Mem, owner: &[u8; 32], key: &[u8]) {
    let g = [42u8; 32];
    push::issue_grant::<Mem, NoLock>(
        d,
        &NoLock,
        "d",
        &push::Kind::Subscribe,
        owner,
        &g,
        key,
        &hash_bytes(&SUB),
    )
    .unwrap();
    push::subscribe(d, &NoLock, "d", &g, key, &SUB, b"endpoint").unwrap();
}

/// True when no authoritative object was committed for this exact key.
pub fn no_data(d: &Mem, cap: &[u8; 32], key: &[u8]) -> bool {
    d.read_all(&data_path("d", &read_cap(cap), key).unwrap())
        .unwrap()
        .is_none()
}

/// Sender-visible wake generations for an exact scope.
pub fn list(d: &Mem, cap: &[u8; 32], key: &[u8]) -> Vec<u64> {
    let w = push::hint_list(d, &NoLock, "d", &SENDER, &read_cap(cap), key).unwrap();
    let n = u32::from_le_bytes(w[..4].try_into().unwrap()) as usize;
    (0..n)
        .map(|i| u64::from_le_bytes(w[4 + i * 8..12 + i * 8].try_into().unwrap()))
        .collect()
}
