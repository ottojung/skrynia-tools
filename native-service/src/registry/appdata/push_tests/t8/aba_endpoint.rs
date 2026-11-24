use super::{MemDisk, caps, push, seed_sender};

#[test]
fn a_b_a_grant_rotation_never_resurrects_old_endpoint() {
    let disk = MemDisk::default();
    let (writer, grant, alice) = caps();
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &[21u8; 32], key);
    let bob = [47u8; 32];
    for subscriber in [bob, alice] {
        push::issue_grant::<MemDisk, crate::NoLock>(
            &disk,
            &crate::NoLock,
            "d",
            &push::Kind::Subscribe,
            &writer,
            &grant,
            key,
            &crate::hash_bytes(&subscriber),
        )
        .unwrap();
    }
    let old = push::sub_fetch(&disk, &crate::NoLock, "d", &[21u8; 32], &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(old[..4].try_into().unwrap()), 0);
    push::subscribe(&disk, &crate::NoLock, "d", &grant, key, &alice, b"fresh").unwrap();
    let fresh = push::sub_fetch(&disk, &crate::NoLock, "d", &[21u8; 32], &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(fresh[..4].try_into().unwrap()), 1);
    assert!(fresh.ends_with(b"fresh"));
}

#[test]
fn reissuing_identical_grant_keeps_existing_subscription_active() {
    let disk = MemDisk::default();
    let (writer, grant, alice) = caps();
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &[21u8; 32], key);
    push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &crate::hash_bytes(&alice),
    )
    .unwrap();
    let out = push::sub_fetch(&disk, &crate::NoLock, "d", &[21u8; 32], &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(out[..4].try_into().unwrap()), 1);
}
