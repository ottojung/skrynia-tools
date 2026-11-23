//! Wake exact-key sender grant and standard seeded-fixture helpers.
use super::{Mem, WAKE, caps, subscribe};
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, read_cap};

pub const SENDER_A: [u8; 32] = [21u8; 32];
pub const SENDER_B: [u8; 32] = [22u8; 32];

pub fn ns() -> [u8; 32] {
    read_cap(&caps().0)
}

pub fn seed_sender(d: &Mem, sender: &[u8]) {
    push::issue_grant::<Mem, NoLock>(
        d,
        &NoLock,
        "d",
        &push::Kind::Sender,
        &caps().0,
        sender,
        WAKE,
        &[0u8; 32],
    )
    .unwrap();
}

/// Durable disk with one wake subscription and one sender grant already seeded.
pub fn seeded() -> Mem {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    d
}
