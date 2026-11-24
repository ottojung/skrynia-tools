use super::revocation_seed::seed_revocation;
use super::{Disk, MemDisk, caps, push};

#[test]
fn subscribe_cannot_write_after_grant_rotates_before_lock() {
    let disk = MemDisk::default();
    let (writer, grant, alice) = caps();
    let key = b"apps/demo/notice";
    let lock = seed_revocation(&disk);
    let got = push::subscribe(&disk, &lock, "d", &grant, key, &alice, b"endpoint");
    assert!(
        got.is_err(),
        "revoked subscriber wrote after issuer rotation"
    );
    let path = push::sub_path("d", &crate::read_cap(&writer), key, &alice);
    assert!(disk.read_all(&path).unwrap().is_none());
}
