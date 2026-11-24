//! A full outbox must not abort an authorized durable object deletion.
use super::{AppPrecond, MemDisk, State, WriteAuth, caps, io, ops, push, seed_sender};

#[test]
fn full_queue_never_aborts_committed_delete() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let initial = b"must-be-deleted".to_vec();
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
    assert!(got.is_ok(), "queue quota must not abort the delete");
    assert_eq!(
        io::read_state(&disk, "d", &ns, key).unwrap(),
        State::Absent,
        "source deletion must commit"
    );
}
