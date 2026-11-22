//! T3: a broad writer cap, a foreign grant, a wrong key or a wrong subscriber
//! can never subscribe — the narrow per-key grant is mandatory.
use super::{MemDisk, caps, push};

#[test]
fn no_matching_grant_means_no_subscribe() {
    let disk = MemDisk::default();
    let (writer, grant, subscriber) = caps();
    let key = b"apps/demo/notice";
    assert!(push::subscribe(&disk, &crate::NoLock, "d", &writer, key, &subscriber, b"x").is_err());
    assert!(push::subscribe(&disk, &crate::NoLock, "d", &grant, key, &subscriber, b"x").is_err());
    push::issue_grant::<MemDisk, ()>(
        &disk,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &[2u8; 32],
    )
    .unwrap();
    assert!(push::subscribe(&disk, &crate::NoLock, "d", &grant, key, &subscriber, b"x").is_err());
    for bad_key in [b"other".as_slice(), b"*", b"notice.*", b"", b"a//b"] {
        assert!(
            push::subscribe(
                &disk,
                &crate::NoLock,
                "d",
                &grant,
                bad_key,
                &subscriber,
                b"x"
            )
            .is_err()
        );
    }
    assert!(push::subscribe(&disk, &crate::NoLock, "d", &grant, key, b"", b"x").is_err());
}
