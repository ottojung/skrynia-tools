use super::{MemDisk, caps, push};

#[test]
fn sender_grant_holder_cannot_overwrite_issuers_scope() {
    let disk = MemDisk::default();
    let (writer, _, _) = caps();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let subject = [0u8; 32];
    push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Sender,
        &writer,
        &sender,
        key,
        &subject,
    )
    .unwrap();
    let hijack = push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Sender,
        &sender,
        &sender,
        key,
        &subject,
    );
    assert!(
        hijack.is_err(),
        "sender re-bound its own scope without issuer"
    );
    assert_eq!(
        push::verify_grant::<MemDisk, ()>(&disk, "d", &push::Kind::Sender, &sender, key, &subject,)
            .unwrap(),
        crate::read_cap(&writer)
    );
}
