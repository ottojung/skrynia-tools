//! Exact authority, generation, corruption and absence negatives for OP27.
use super::support::setup;
use skrynia_native_service::wire::{Request, STATUS_ERR, STATUS_OK};

#[test]
fn op27_negatives_fail_closed_without_mutation() {
    let f = setup();
    let exp = f.expected();
    let before = f.snapshot();
    assert_eq!(
        f.status(b"apps/demo/other", &f.subscriber, &f.grant, exp.clone()),
        (STATUS_ERR, vec![9])
    );
    assert_eq!(
        f.status(&f.key, b"subscriber-B", &f.grant, exp.clone()),
        (STATUS_ERR, vec![9])
    );
    assert_eq!(
        f.status(&f.key, &f.subscriber, &[8u8; 32], exp.clone()),
        (STATUS_ERR, vec![9])
    );
    assert_eq!(
        f.status(&f.key, &f.subscriber, &f.grant, vec![9u8; 32]),
        (STATUS_ERR, vec![8])
    );
    assert_eq!(f.snapshot(), before);
    f.reg
        .push_unsubscribe(
            &f.disk,
            &skrynia_native_service::NoLock,
            &f.grant,
            &f.key,
            &f.subscriber,
        )
        .unwrap();
    let expiry = f.status(&f.key, &f.subscriber, &f.grant, exp);
    assert_eq!(expiry, (STATUS_OK, vec![0]));
}
