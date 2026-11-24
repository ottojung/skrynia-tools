//! A held mutation lock makes queries fail closed as busy.
use super::lock_support::{Mem, Stub};
use skrynia_native_service::MutLock;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::index;
use skrynia_native_service::mlock::StdLock;
use skrynia_native_service::{
    handle::handle,
    reg::Registry,
    wdec::decode_res,
    wenc::encode_req,
    wire::{AdminRequest, Request},
};

#[test]
fn list_programs_does_not_recover_under_another_mutation_lock() {
    let d = Mem::default();
    let l = StdLock::default();
    let r = Registry::open(&d, "reg").unwrap();
    let id: [u8; 32] = r
        .register(&d, &l, &[5; 32], b"code".to_vec())
        .unwrap()
        .try_into()
        .unwrap();
    let s = index::scope("reg");
    index::write_pending(&d, s, id).unwrap();
    assert!(l.try_acquire().unwrap());
    let before = d.0.borrow().clone();
    let q = encode_req(&Request::Admin(AdminRequest::ListPrograms {
        after: None,
        limit: 64,
    }));
    let result = decode_res(&handle(&r, &d, &l, &Stub, &Stub, &q)).unwrap();
    assert_eq!(
        *d.0.borrow(),
        before,
        "query changed durable state while lock held"
    );
    assert_eq!(
        (result.status, result.body),
        (1, vec![11]),
        "must return explicit busy"
    );
    l.release().unwrap();
    let result = decode_res(&handle(&r, &d, &l, &Stub, &Stub, &q)).unwrap();
    assert_eq!(result.status, 0);
    assert!(index::take_pending(&d, s).unwrap().is_none());
}
