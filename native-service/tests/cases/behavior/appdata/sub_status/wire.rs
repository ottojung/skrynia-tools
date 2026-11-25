//! Encoded OP27 end-to-end: exact response bytes and frame validation.
use super::support::setup;
use skrynia_native_service::{
    wdec::decode_req,
    wenc::encode_req,
    wire::{OP_SUB_STATUS, Request, STATUS_ERR, STATUS_OK},
};

#[test]
fn encoded_op27_active_response_is_exact_and_read_only() {
    let f = setup();
    let req = Request::SubStatus(
        f.grant.to_vec(),
        f.key.clone(),
        f.subscriber.clone(),
        f.expected(),
    );
    let frame = encode_req(&req);
    assert_eq!(frame[0], 1);
    assert_eq!(frame[1], OP_SUB_STATUS);
    assert_eq!(decode_req(&frame).unwrap(), req);
    let before = f.snapshot();
    let out = f.call(req);
    assert_eq!(out.status, STATUS_OK);
    assert_eq!(out.body.len(), 33);
    assert_eq!(out.body[0], 1);
    assert_eq!(&out.body[1..], f.expected().as_slice());
    assert_eq!(f.snapshot(), before, "OP27 must not mutate durable state");
}

#[test]
fn trailing_and_truncated_op27_frames_are_rejected() {
    let f = setup();
    let frame = encode_req(&Request::SubStatus(
        f.grant.to_vec(),
        f.key.clone(),
        f.subscriber.clone(),
        f.expected(),
    ));
    let mut tailed = frame.clone();
    tailed.push(0);
    assert!(decode_req(&tailed).is_err());
    assert!(decode_req(&frame[..frame.len() - 1]).is_err());
    let bad = f.call(Request::SubStatus(
        f.grant.to_vec(),
        f.key.clone(),
        f.subscriber.clone(),
        vec![0u8; 31],
    ));
    assert_eq!((bad.status, bad.body), (STATUS_ERR, vec![9]));
}
