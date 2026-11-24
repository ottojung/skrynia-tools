//! Corrupt notification counter stays visible and fail-closed, but never
//! aborts an authorized source CAS.
use super::Disk;
use super::{AppPrecond, MemDisk, State, WriteAuth, caps, io, ops, push, seed_sender};

#[test]
fn corrupt_notification_counter_does_not_abort_committed_cas() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let path = push::hint_counter_path("d", &ns, key);
    disk.write_atomic(&path, b"bad-state").unwrap();
    let got = ops::put(
        &disk,
        &crate::NoLock,
        "d",
        WriteAuth::External(&writer),
        key,
        &AppPrecond::Absent,
        b"value",
    );
    assert!(
        got.is_ok(),
        "notification corruption is not a data-plane failure"
    );
    assert_eq!(
        io::read_state(&disk, "d", &ns, key).unwrap(),
        State::Present(b"value".to_vec())
    );
    assert_eq!(
        disk.read_all(&path).unwrap(),
        Some(b"bad-state".to_vec()),
        "corrupt notification state remains visible for recovery"
    );
}
