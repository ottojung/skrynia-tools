use super::util;
use super::{MemDisk, NoLock, Registry, auth};
use skrynia_program0::wire::{AppPrecond, Request, STATUS_ERR, STATUS_OK};

pub fn run() {
    let (d, l) = (MemDisk::default(), NoLock);
    use skrynia_program0::cap::Disk;
    d.write_atomic("reg/operator.cap", &[85u8; 32]).unwrap();
    let r = Registry::open(&d, "reg").unwrap();
    let a = vec![2u8; 32];
    let iid = auth::read_cap_vec(&a);
    let h = AppPrecond::Present(auth::hash_bytes(b"code").to_vec());
    let reg_req = Request::Register {
        operator_cap: vec![85; 32],
        admin_cap: a.clone(),
        code: b"code".to_vec(),
    };
    assert_eq!(util::call(&r, &d, &l, reg_req).body, iid);
    let got = util::call(
        &r,
        &d,
        &l,
        Request::Lookup {
            invoke_id: iid.clone(),
        },
    );
    assert_eq!(got.body, b"code");
    let bad = Request::Revoke {
        admin_cap: vec![0; 32],
        expect: h.clone(),
    };
    assert_eq!(util::call(&r, &d, &l, bad).status, STATUS_ERR);
    assert_eq!(
        util::call(
            &r,
            &d,
            &l,
            Request::Revoke {
                admin_cap: a,
                expect: h
            }
        )
        .status,
        STATUS_OK
    );
    let inv = Request::Invoke {
        invoke_id: iid,
        input: vec![],
    };
    assert_eq!(util::call(&r, &d, &l, inv).status, STATUS_ERR);
}
