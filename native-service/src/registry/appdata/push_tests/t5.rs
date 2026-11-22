//! T5: only a namespace writer may issue grants, and a disk remove failure
//! during unsubscribe is surfaced instead of being silently swallowed.
use super::support::FailRemove;
use super::{MemDisk, caps, push};

#[test]
fn only_writer_issues_and_remove_errors_surface() {
    let disk = MemDisk::default();
    let (writer, grant, subscriber) = caps();
    let key = b"k";
    let bad = push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &[0u8; 3],
        &grant,
        key,
        &[1u8; 32],
    );
    assert!(bad.is_err());
    let wildcard = push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        b"*",
        &[1u8; 32],
    );
    assert!(wildcard.is_err());
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
    push::subscribe(&disk, &crate::NoLock, "d", &grant, key, &subscriber, b"ep").unwrap();
    let failing = FailRemove(&disk);
    assert!(push::unsubscribe(&failing, &crate::NoLock, "d", &grant, key, &subscriber).is_err());
    assert!(push::unsubscribe(&disk, &crate::NoLock, "d", &grant, key, &subscriber).is_ok());
}
