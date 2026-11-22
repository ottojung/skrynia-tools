use super::{MemDisk, caps, push};

#[test]
fn issuer_can_replace_subject_but_not_namespace() {
    let disk = MemDisk::default();
    let (issuer, grant, alice) = caps();
    let key = b"apps/demo/notice";
    let bob = [12u8; 32];
    let a = crate::hash_bytes(&alice);
    let b = crate::hash_bytes(&bob);
    for subject in [a, b] {
        push::issue_grant::<MemDisk, crate::NoLock>(
            &disk,
            &crate::NoLock,
            "d",
            &push::Kind::Subscribe,
            &issuer,
            &grant,
            key,
            &subject,
        )
        .unwrap();
    }
    assert!(
        push::verify_grant::<MemDisk, ()>(&disk, "d", &push::Kind::Subscribe, &grant, key, &a,)
            .is_err()
    );
    assert_eq!(
        push::verify_grant::<MemDisk, ()>(&disk, "d", &push::Kind::Subscribe, &grant, key, &b,)
            .unwrap(),
        crate::read_cap(&issuer)
    );
}
