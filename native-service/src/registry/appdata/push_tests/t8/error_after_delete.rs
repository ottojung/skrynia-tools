//! Full outbox must reject CAS delete before durable object deletion.
use super::{AppPrecond, MemDisk, State, WriteAuth, caps, io, ops, push, seed_sender};

#[test]
fn full_queue_cannot_report_failed_delete_after_commit() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let initial = b"must-survive".to_vec();
    ops::put(
        &disk,
        &crate::NoLock,
        "d",
        WriteAuth::External(&writer),
        key,
        &AppPrecond::Absent,
        &initial,
    )
    .unwrap();
    for i in 2..push::MAX_HINTS {
        push::enqueue_hints(&disk, "d", &ns, key, Some([i as u8; 32])).unwrap();
    }
    let got = ops::delete(
        &disk,
        &crate::NoLock,
        "d",
        WriteAuth::External(&writer),
        key,
        &crate::hash_bytes(&initial),
    );
    assert!(got.is_err(), "queue quota must fail before deleting");
    assert_eq!(
        io::read_state(&disk, "d", &ns, key).unwrap(),
        State::Present(initial)
    );
}
