//! Grant rotation and A-B-A generation mismatch.
use super::support::setup;
use skrynia_native_service::wire::STATUS_ERR;

#[test]
fn op27_rotation_and_aba_generation_mismatch() {
    let f = setup();
    let exp = f.expected();
    let no_lock = skrynia_native_service::NoLock;
    f.reg
        .push_grant(
            &f.disk,
            &no_lock,
            &[3u8; 32],
            &f.grant,
            &f.key,
            b"subscriber-B",
        )
        .unwrap();
    assert_eq!(
        f.status(&f.key, &f.subscriber, &f.grant, exp.clone()),
        (STATUS_ERR, vec![9])
    );
    f.reg
        .push_grant(
            &f.disk,
            &no_lock,
            &[3u8; 32],
            &f.grant,
            &f.key,
            &f.subscriber,
        )
        .unwrap();
    assert_eq!(
        f.status(&f.key, &f.subscriber, &f.grant, exp.clone()),
        (STATUS_ERR, vec![9])
    );
}
