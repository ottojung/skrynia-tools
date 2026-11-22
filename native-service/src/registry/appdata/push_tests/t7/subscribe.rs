use super::{MemDisk, caps, push};

#[test]
fn grant_holder_cannot_overwrite_issuers_subscribe_grant() {
    let disk = MemDisk::default();
    let (writer, grant, subscriber) = caps();
    let key = b"apps/demo/notice";
    let subject = crate::hash_bytes(&subscriber);
    push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &subject,
    )
    .unwrap();
    let hijack = push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &grant,
        &grant,
        key,
        &subject,
    );
    assert!(
        hijack.is_err(),
        "narrow-grant holder changed issuer namespace"
    );
    assert_eq!(
        push::verify_grant::<MemDisk, ()>(
            &disk,
            "d",
            &push::Kind::Subscribe,
            &grant,
            key,
            &subject,
        )
        .unwrap(),
        crate::read_cap(&writer)
    );
}
