use super::revocation_seed::seed_revocation;
use super::{Disk, MemDisk, caps, push};

#[test]
fn unsubscribe_cannot_delete_after_grant_rotates_before_lock() {
    let disk = MemDisk::default();
    let (writer, grant, alice) = caps();
    let key = b"apps/demo/notice";
    let lock = seed_revocation(&disk);
    push::subscribe(&disk, &crate::NoLock, "d", &grant, key, &alice, b"endpoint").unwrap();
    let path = push::sub_path("d", &crate::read_cap(&writer), key, &alice);
    let got = push::unsubscribe(&disk, &lock, "d", &grant, key, &alice);
    assert!(
        got.is_err(),
        "revoked subscriber deleted after issuer rotation"
    );
    assert!(
        disk.read_all(&path)
            .unwrap()
            .unwrap()
            .ends_with(b"endpoint")
    );
}
