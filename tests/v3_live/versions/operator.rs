use super::fixture::server;
use crate::{guest, wire};
use skrynia_program0::{
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, STATUS_ERR},
};
#[test]
#[ignore = "requires independently compiled operator-provisioned Program 0"]
fn retired_register_wire_opcode_and_wrong_operator_cap_are_refused() {
    let (server, _, _) = server();
    let unauthorized = Request::Register {
        operator_cap: vec![0u8; 32],
        admin_cap: vec![33u8; 32],
        code: guest::byte(90),
    };
    assert!(wire::call(&server.url, unauthorized.clone()).is_err());
    let mut old_wire = encode_req(&unauthorized);
    old_wire[1] = 1;
    let res = reqwest::blocking::Client::new()
        .post(&server.url)
        .header("content-type", "application/octet-stream")
        .body(old_wire)
        .send()
        .unwrap();
    assert_eq!(res.status(), 200);
    let frame = decode_res(&res.bytes().unwrap()).unwrap();
    assert_eq!(frame.status, STATUS_ERR);
    assert_eq!(frame.body, [1], "legacy registration must fail parsing");
    let valid = Request::Register {
        operator_cap: vec![85u8; 32],
        admin_cap: vec![33u8; 32],
        code: guest::byte(90),
    };
    assert_eq!(wire::call(&server.url, valid).unwrap().len(), 32);
}
