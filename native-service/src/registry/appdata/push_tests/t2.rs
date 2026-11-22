//! T2: durable grant/subscribe/unregister across a simulated registry reopen.
use super::Disk;
use super::{MemDisk, caps, push};

#[test]
fn register_survives_reopen_and_unregister_removes() {
    let disk = MemDisk::default();
    let (writer, grant, subscriber) = caps();
    let key = b"apps/demo/notice";
    let subject = crate::hash_bytes(&subscriber);
    push::issue_grant::<MemDisk, ()>(
        &disk,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &subject,
    )
    .unwrap();
    push::subscribe(
        &disk,
        &crate::NoLock,
        "d",
        &grant,
        key,
        &subscriber,
        b"endpoint",
    )
    .unwrap();
    // Reopen: fresh facade, same durable disk.
    push::subscribe(
        &disk,
        &crate::NoLock,
        "d",
        &grant,
        key,
        &subscriber,
        b"endpoint-2",
    )
    .unwrap();
    let file = push::sub_path("d", &crate::read_cap(&writer), key, &subscriber);
    assert_eq!(disk.read_all(&file).unwrap().unwrap(), b"endpoint-2");
    push::unsubscribe(&disk, &crate::NoLock, "d", &grant, key, &subscriber).unwrap();
    assert!(disk.read_all(&file).unwrap().is_none());
    assert!(push::unsubscribe(&disk, &crate::NoLock, "d", &grant, key, &subscriber).is_err());
}
