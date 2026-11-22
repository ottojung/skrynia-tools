use super::{Disk, MemDisk, caps, push, seed_sender};

#[test]
fn legacy_unbound_endpoint_record_is_not_delivered() {
    let disk = MemDisk::default();
    let (writer, _, alice) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&writer);
    seed_sender(&disk, &sender, key);
    let file = push::sub_path("d", &ns, key, &alice);
    disk.write_atomic(&file, b"legacy-raw-endpoint").unwrap();
    let out = push::sub_fetch(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(out[..4].try_into().unwrap()), 0);
}
