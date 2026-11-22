use super::{MemDisk, caps, push, seed_sender};

#[test]
fn rotated_grant_revokes_existing_endpoint_delivery() {
    let disk = MemDisk::default();
    let (writer, grant, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let first = push::sub_fetch(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(first[..4].try_into().unwrap()), 1);
    let bob = [47u8; 32];
    push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &crate::hash_bytes(&bob),
    )
    .unwrap();
    let now = push::sub_fetch(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(now[..4].try_into().unwrap()), 0);
}
