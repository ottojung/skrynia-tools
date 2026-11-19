mod cases;
mod cases_admin;
use skrynia_program0::wdec::{decode_req, decode_res};
use skrynia_program0::wenc::{encode_req, encode_res};
use skrynia_program0::wire::{Response, STATUS_OK};

#[test]
fn req_roundtrip() {
    for c in cases::req_cases()
        .into_iter()
        .chain(cases_admin::req_cases())
    {
        let b = encode_req(&c);
        assert_eq!(decode_req(&b).unwrap(), c);
    }
}

#[test]
fn res_roundtrip() {
    let r = Response {
        status: STATUS_OK,
        body: vec![42, 43],
    };
    let b = encode_res(&r);
    assert_eq!(decode_res(&b).unwrap(), r);
}
