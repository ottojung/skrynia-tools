//! Lock contention and lock-release failure map to busy/IO, never revoked.
use super::disk::{Eval0, Rng};
use skrynia_native_service::{
    MutLock,
    handle::handle,
    mlock::StdLock,
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, STATUS_ERR},
};

#[test]
fn op27_held_lock_is_busy_without_mutation() {
    let f = super::support::setup();
    let lock = StdLock::default();
    assert!(lock.try_acquire().unwrap());
    let before = f.snapshot();
    let req = Request::SubStatus(
        f.grant.to_vec(),
        f.key.clone(),
        f.subscriber.clone(),
        f.expected(),
    );
    let out = decode_res(&handle(
        &f.reg,
        &f.disk,
        &lock,
        &Rng,
        &Eval0,
        &encode_req(&req),
    ))
    .unwrap();
    assert_eq!((out.status, out.body), (STATUS_ERR, vec![11]));
    assert_eq!(f.snapshot(), before);
    lock.release().unwrap();
}
