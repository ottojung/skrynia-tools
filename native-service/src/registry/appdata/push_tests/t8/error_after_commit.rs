//! A full per-key hint queue is degraded notification, not a data-plane error.
use super::Disk;
use super::{AppPrecond, MemDisk, State, WriteAuth, caps, io, ops, push, seed_sender};

#[test]
fn full_notification_queue_never_aborts_committed_put() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    for i in 1..push::MAX_HINTS {
        push::enqueue_hints(&disk, "d", &ns, key, Some([i as u8; 32])).unwrap();
    }
    let result = ops::put(
        &disk,
        &crate::NoLock,
        "d",
        WriteAuth::External(&writer),
        key,
        &AppPrecond::Absent,
        b"committed-value",
    );
    assert!(
        result.is_ok(),
        "notification quota must not abort the data CAS"
    );
    assert_eq!(
        io::read_state(&disk, "d", &ns, key).unwrap(),
        State::Present(b"committed-value".to_vec()),
        "source value must commit despite a full hint queue"
    );
    let hints = disk.list(&push::hints_root("d", &ns, key)).unwrap();
    assert!(hints.len() <= push::MAX_HINTS, "hint queue stays bounded");
}
