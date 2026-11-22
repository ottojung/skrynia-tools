use super::revocation_lock::RotateAtAcquire;
use super::{Disk, MemDisk, caps, push};

pub fn seed_revocation(disk: &MemDisk) -> RotateAtAcquire<'_> {
    let (writer, grant, alice) = caps();
    let key = b"apps/demo/notice";
    let bob = [42u8; 32];
    for subscriber in [bob, alice] {
        push::issue_grant::<MemDisk, crate::NoLock>(
            disk,
            &crate::NoLock,
            "d",
            &push::Kind::Subscribe,
            &writer,
            &grant,
            key,
            &crate::hash_bytes(&subscriber),
        )
        .unwrap();
        if subscriber == bob {
            // This is the authentic issuer-written state after revocation.
            let suffix = disk.list("d/pushgrant").unwrap().pop().unwrap();
            let path = format!("d/pushgrant/{suffix}");
            let next = disk.read_all(&path).unwrap().unwrap();
            return seed_original_then_rotate(disk, &writer, &grant, &alice, key, path, next);
        }
    }
    unreachable!()
}
fn seed_original_then_rotate<'a>(
    disk: &'a MemDisk,
    writer: &[u8],
    grant: &[u8],
    alice: &[u8],
    key: &[u8],
    path: String,
    next: Vec<u8>,
) -> RotateAtAcquire<'a> {
    push::issue_grant::<MemDisk, crate::NoLock>(
        disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        writer,
        grant,
        key,
        &crate::hash_bytes(alice),
    )
    .unwrap();
    RotateAtAcquire::new(disk, path, next)
}
