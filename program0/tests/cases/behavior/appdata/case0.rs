use super::eval::GuestEval;
use crate::cases::support::{MemDisk, Rng};
use skrynia_program0::handle::handle;
use skrynia_program0::reg::Registry;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::{Request, STATUS_OK};

#[test]
fn app_data_and_isolated_guest_callbacks() {
    let d = MemDisk::default();
    let r = Registry::open(&d, "p").unwrap();
    let id = r.register(&d, &Rng(99), b"guest".to_vec()).unwrap();
    let put = handle(
        &r,
        &d,
        &Rng(0),
        &GuestEval,
        &encode_req(&Request::AppPut {
            app: id.clone(),
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        }),
    );
    assert_eq!(decode_res(&put).unwrap().status, STATUS_OK);
    let inv = handle(
        &r,
        &d,
        &Rng(0),
        &GuestEval,
        &encode_req(&Request::Invoke {
            id: id.clone(),
            input: id.clone(),
        }),
    );
    assert_eq!(decode_res(&inv).unwrap().body, b"done");
    let get = handle(
        &r,
        &d,
        &Rng(0),
        &GuestEval,
        &encode_req(&Request::AppGet {
            app: id,
            key: b"k".to_vec(),
        }),
    );
    assert_eq!(decode_res(&get).unwrap().body, b"v+cb");
}
