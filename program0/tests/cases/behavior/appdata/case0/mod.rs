mod util;
use super::eval::GuestEval;
use crate::cases::support::{MemDisk, NoLock, Rng};
use skrynia_program0::handle::handle;
use skrynia_program0::reg::Registry;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::{AppPrecond, Request, STATUS_OK};

#[test]
fn app_data_caps_cas_and_guest_callback_isolation() {
    let d = MemDisk::default();
    let lock = NoLock;
    let r = Registry::open(&d, "p").unwrap();
    let admin = [99u8; 32];
    let iid = r.register(&d, &lock, &admin, b"guest".to_vec()).unwrap();
    let read_cap = iid.clone();
    let put = util::call(
        &r,
        &d,
        &lock,
        Request::AppPut {
            write_cap: admin.to_vec(),
            key: b"k".to_vec(),
            expect: AppPrecond::Absent,
            value: b"v".to_vec(),
        },
    );
    assert_eq!(put.status, STATUS_OK);
    let inv = util::call(
        &r,
        &d,
        &lock,
        Request::Invoke {
            invoke_id: iid.clone(),
            input: iid.clone(),
        },
    );
    assert_eq!(inv.body, b"done");
    let get = util::call(
        &r,
        &d,
        &lock,
        Request::AppGet {
            read_cap,
            key: b"k".to_vec(),
        },
    );
    assert_eq!(get.body, b"v+cb");
}
