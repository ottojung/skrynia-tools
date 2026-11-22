use super::{MemDisk, caps, push, seed_sender};

#[test]
fn acknowledged_notification_sequence_must_never_be_reused() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let list = push::hint_list(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    let first = u64::from_le_bytes(list[4..12].try_into().unwrap());
    push::hint_ack(&disk, &crate::NoLock, "d", &sender, &ns, key, first).unwrap();
    push::enqueue_hints(&disk, "d", &ns, key, Some([9u8; 32])).unwrap();
    let new_list = push::hint_list(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    let second = u64::from_le_bytes(new_list[4..12].try_into().unwrap());
    assert_ne!(first, second, "a repeated ACK could delete a newer hint");
    push::hint_ack(&disk, &crate::NoLock, "d", &sender, &ns, key, first).unwrap();
    assert!(push::hint_read(&disk, &crate::NoLock, "d", &sender, &ns, key, second).is_ok());
}
