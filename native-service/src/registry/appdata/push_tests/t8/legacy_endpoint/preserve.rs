use super::{Disk, MemDisk, caps, push, seed_sender};

#[test]
fn legacy_records_remain_intact_until_explicit_migration() {
    let disk = MemDisk::default();
    let (writer, grant, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let old_subscriber = [99u8; 32];
    let old_file = push::sub_path("d", &ns, key, &old_subscriber);
    disk.write_atomic(&old_file, b"legacy-unmigrated-endpoint")
        .unwrap();
    let new_subscriber = [47u8; 32];
    push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &crate::hash_bytes(&new_subscriber),
    )
    .unwrap();
    push::subscribe(
        &disk,
        &crate::NoLock,
        "d",
        &grant,
        key,
        &new_subscriber,
        b"fresh-endpoint",
    )
    .unwrap();
    assert_eq!(
        disk.read_all(&old_file).unwrap().unwrap(),
        b"legacy-unmigrated-endpoint",
        "legacy data must not be destroyed implicitly"
    );
    let out = push::sub_fetch(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(out[..4].try_into().unwrap()), 1);
}
