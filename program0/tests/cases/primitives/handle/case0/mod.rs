mod util;
use super::{CatEval, MemDisk, NoLock, Registry, Rng, auth, decode_res, encode_req, handle};
use skrynia_program0::wire::{AppPrecond, Request, STATUS_OK};

#[test]
fn wire_register_set_invoke() {
    let (d, l) = (MemDisk::default(), NoLock);
    use skrynia_program0::cap::Disk;
    d.write_atomic("reg/operator.cap", &[85u8; 32]).unwrap();
    let r = Registry::open(&d, "reg").unwrap();
    let a = vec![7u8; 32];
    let iid = auth::read_cap_vec(&a);
    let reg_req = Request::Register {
        operator_cap: vec![85; 32],
        admin_cap: a.clone(),
        code: b"a".to_vec(),
    };
    assert_eq!(util::call(&r, &d, &l, reg_req).body, iid);
    let inv1 = Request::Invoke {
        invoke_id: iid.clone(),
        input: b"1".to_vec(),
    };
    assert_eq!(util::call(&r, &d, &l, inv1).body, b"a:1");
    let set = Request::Set {
        admin_cap: a,
        expect: AppPrecond::Present(auth::hash_bytes(b"a").to_vec()),
        code: b"b".to_vec(),
    };
    util::call(&r, &d, &l, set);
    let inv2 = Request::Invoke {
        invoke_id: iid,
        input: b"2".to_vec(),
    };
    let out = util::call(&r, &d, &l, inv2);
    assert_eq!(out.status, STATUS_OK);
    assert_eq!(out.body, b"b:2");
}
