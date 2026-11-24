//! Issue 5z exact-pin wake fixtures over the real ordinary P1 source.
use super::{Mem, SENDER_A, WAKE, caps, data_exists, hints, ns, read_ok, seed_sender, subscribe};
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;

/// The single exact key Along registers at the RED pin.
pub const A: &[u8] = b"traveller-tes-v1";
/// The second traveller key that the RED pin never wakes.
pub const B: &[u8] = b"traveller-vitalik-v1";

/// One exact-key subscription to A plus one sender at the wake key.
pub fn observed() -> Mem {
    let d = Mem::default();
    subscribe(&d, A);
    subscribe(&d, super::WAKE);
    seed_sender(&d, &SENDER_A);
    d
}

pub fn put(d: &Mem, key: &[u8]) {
    ops::put(
        d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        key,
        &AppPrecond::Absent,
        b"v",
    )
    .unwrap();
}

/// Sender-visible generations at the designated exact wake key.
pub fn wake_gens(d: &Mem) -> Vec<u64> {
    let w = push::hint_list(d, &NoLock, "d", &SENDER_A, &ns(), WAKE).unwrap();
    let n = u32::from_le_bytes(w[..4].try_into().unwrap()) as usize;
    (0..n)
        .map(|i| u64::from_le_bytes(w[4 + i * 8..12 + i * 8].try_into().unwrap()))
        .collect()
}

/// Write with an explicit capability; used for foreign-key negatives.
pub fn put_as(d: &Mem, cap: &[u8], key: &[u8]) -> bool {
    let auth = WriteAuth::External(cap);
    ops::put(d, &NoLock, "d", auth, key, &AppPrecond::Absent, b"v").is_ok()
}

/// Subscribing A only must never receive B; the configured wake key must.
pub fn two_key_red() {
    let d = observed();
    put(&d, B);
    assert_eq!(hints(&d, A), 0);
    assert_eq!(wake_gens(&d), vec![1]);
    assert!(data_exists(&d, B) && read_ok(&d, 1));
}
