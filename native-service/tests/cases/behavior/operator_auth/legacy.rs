use super::fixture::install;
use skrynia_native_service::{wdec::decode_req, wenc::encode_req, wire::VERSION};

#[test]
fn retired_unsecured_register_opcode_is_not_reinterpreted_as_authorized() {
    let req = install(&[11; 32]);
    let encoded = encode_req(&req);
    assert_eq!(encoded[0], VERSION);
    assert_eq!(encoded[1], 15);
    let mut legacy = encoded.clone();
    legacy[1] = 1;
    assert!(decode_req(&legacy).is_err());
}
