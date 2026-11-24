use super::{Disk, MemDisk, caps, push, seed_sender};

#[test]
fn rotating_grants_cannot_exhaust_quota_with_inactive_endpoints() {
    let disk = MemDisk::default();
    let (writer, grant, _) = caps();
    let key = b"apps/demo/notice";
    let sender = [21u8; 32];
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    for i in 1..=80u8 {
        let sub = [i; 32];
        push::issue_grant::<MemDisk, crate::NoLock>(
            &disk,
            &crate::NoLock,
            "d",
            &push::Kind::Subscribe,
            &writer,
            &grant,
            key,
            &crate::hash_bytes(&sub),
        )
        .unwrap();
        push::subscribe(&disk, &crate::NoLock, "d", &grant, key, &sub, b"endpoint").unwrap();
    }
    let out = push::sub_fetch(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(out[..4].try_into().unwrap()), 1);
    assert!(disk.list(&push::subs_dir("d", &ns, key)).unwrap().len() <= push::MAX_SUBS);
}
