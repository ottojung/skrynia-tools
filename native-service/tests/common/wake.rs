//! Wake-generation helpers over the real P1 source and in-memory durable Disk.
use super::{Mem, WAKE, caps};
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
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

pub fn pending(d: &Mem) -> Vec<u64> {
    let mut out: Vec<u64> = d
        .list(&push::hints_root("d", &ns(), WAKE))
        .unwrap()
        .iter()
        .filter_map(|n| n.strip_suffix(".bin")?.parse().ok())
        .collect();
    out.sort_unstable();
    out
}

pub fn ack(d: &Mem, sender: &[u8], seq: u64) {
    push::hint_ack(d, &NoLock, "d", sender, &ns(), WAKE, seq).unwrap();
}

pub fn put_ok(d: &Mem, key: &[u8]) -> bool {
    ops::put(
        d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        key,
        &AppPrecond::Absent,
        b"v",
    )
    .is_ok()
}
