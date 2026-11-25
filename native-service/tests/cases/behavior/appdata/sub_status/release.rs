//! Lock-release failure maps to IO7.
use super::disk::{Eval0, Rng};
use skrynia_native_service::{
    MutLock,
    handle::handle,
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, STATUS_ERR},
};
struct BadRelease;
impl MutLock for BadRelease {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        Ok(true)
    }
    fn release(&self) -> Result<(), ()> {
        Err(())
    }
}

#[test]
fn op27_release_failure_is_io_not_revoked() {
    let f = super::support::setup();
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
        &BadRelease,
        &Rng,
        &Eval0,
        &encode_req(&req),
    ))
    .unwrap();
    assert_eq!((out.status, out.body), (STATUS_ERR, vec![7]));
    assert_eq!(f.snapshot(), before);
}
