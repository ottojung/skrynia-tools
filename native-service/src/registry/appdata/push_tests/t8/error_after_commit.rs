use super::{AppPrecond, MemDisk, State, WriteAuth, caps, io, ops, push, seed_sender};

#[test]
fn full_notification_queue_cannot_report_failed_put_after_commit() {
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
    assert!(result.is_err(), "quota must block BEFORE changing data");
    assert_eq!(
        io::read_state(&disk, "d", &ns, key).unwrap(),
        State::Absent,
        "failed CAS cannot have committed value"
    );
}
