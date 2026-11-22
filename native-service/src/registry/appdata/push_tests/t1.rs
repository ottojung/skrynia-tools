//! T1: exact key accepted; wildcard, prefix, empty, malformed and foreign
//! scope rejected by the grammar and by grant verification.
use super::{MemDisk, caps, path, push};

#[test]
fn exact_key_grammar_negatives() {
    assert!(path::valid_key(b"apps/demo/notice"));
    for bad in [
        b"" as &[u8],
        b"*",
        b"notice.*",
        b"apps/long/*",
        b"notice/",
        b".hidden",
        b"a b",
        b"a//b",
        b"..",
        &[0u8],
    ] {
        assert!(!path::valid_key(bad), "accepted malformed key: {bad:?}");
    }
}

#[test]
fn foreign_writer_cannot_reuse_another_namespaces_grant() {
    let disk = MemDisk::default();
    let (writer, grant, _sub) = caps();
    let key = b"apps/demo/notice";
    let subject = [1u8; 32];
    let kind = push::Kind::Subscribe;
    push::issue_grant::<MemDisk, crate::NoLock>(
        &disk,
        &crate::NoLock,
        "d",
        &kind,
        &writer,
        &grant,
        key,
        &subject,
    )
    .unwrap();
    let verify = |cap: &[u8]| {
        push::verify_grant::<MemDisk, ()>(&disk, "d", &kind, cap, key, &subject).is_ok()
    };
    assert!(verify(&grant));
    assert!(!verify(&[8u8; 32]));
}
