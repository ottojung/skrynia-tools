//! Decoder and encoder agreement for cross-namespace capability requests.
use skrynia_native_service::wdec::decode_req;
use skrynia_native_service::wenc::encode_req;
use skrynia_native_service::wire::{AppPrecond, Request};

fn roundtrip(req: Request, op: u8) {
    let encoded = encode_req(&req);
    assert_eq!(encoded[1], op);
    assert_eq!(decode_req(&encoded), Ok(req));
    assert!(decode_req(&encoded[..encoded.len() - 1]).is_err());
}

#[test]
fn exact_wake_capability_wire_roundtrips_and_rejects_truncation() {
    roundtrip(
        Request::PushWakeGrant(
            vec![1; 32],
            vec![2; 32],
            b"skrynia-wake-v1".to_vec(),
            vec![3; 32],
        ),
        24,
    );
    roundtrip(
        Request::AppPutWake(
            vec![4; 32],
            b"traveller-tes-v1".to_vec(),
            AppPrecond::Absent,
            b"data".to_vec(),
            vec![5; 32],
            b"skrynia-wake-v1".to_vec(),
        ),
        25,
    );
    roundtrip(
        Request::AppDeleteWake(
            vec![4; 32],
            b"thread-a-tes-v1".to_vec(),
            vec![6; 32],
            vec![5; 32],
            b"skrynia-wake-v1".to_vec(),
        ),
        26,
    );
}
