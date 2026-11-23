//! Wake-slot sender-surface helpers over the real P1 source.
use super::caps;
use super::sender::{SENDER_A, ns};
use skrynia_native_service::NoLock;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;

/// Sender-visible wake seqs through the unchanged P1 hint_list wire format.
pub fn list<D: Disk>(d: &D) -> Vec<u64>
where
    D::Err: std::fmt::Debug,
{
    let wire = push::hint_list(d, &NoLock, "d", &SENDER_A, &ns(), super::WAKE).unwrap();
    let count = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
    (0..count)
        .map(|i| u64::from_le_bytes(wire[4 + i * 8..12 + i * 8].try_into().unwrap()))
        .collect()
}

pub fn ack<D: Disk>(d: &D, sender: &[u8], seq: u64) {
    assert!(push::hint_ack(d, &NoLock, "d", sender, &ns(), super::WAKE, seq).is_ok());
}

pub fn read_ok<D: Disk>(d: &D, seq: u64) -> bool {
    push::hint_read(d, &NoLock, "d", &SENDER_A, &ns(), super::WAKE, seq).is_ok()
}

pub fn put_ok<D: Disk>(d: &D, key: &[u8]) -> bool {
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
