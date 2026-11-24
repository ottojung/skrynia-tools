use skrynia_native_service::protocol::{decoder, wire};
use skrynia_tools::wire::native;

#[test]
fn client_frames_decode_in_the_actual_service() {
    let cap = [42; 32];
    let hash = native::digest(b"old");
    let key = "app/key";
    let cases = [
        (
            native::get(&cap, key).unwrap(),
            wire::Request::AppGet {
                read_cap: cap.to_vec(),
                key: key.as_bytes().to_vec(),
            },
        ),
        (
            native::put(&cap, key, None, b"value").unwrap(),
            wire::Request::AppPut {
                write_cap: cap.to_vec(),
                key: key.as_bytes().to_vec(),
                expect: wire::AppPrecond::Absent,
                value: b"value".to_vec(),
            },
        ),
        (
            native::put(&cap, key, Some(&hash), b"value").unwrap(),
            wire::Request::AppPut {
                write_cap: cap.to_vec(),
                key: key.as_bytes().to_vec(),
                expect: wire::AppPrecond::Present(hash.to_vec()),
                value: b"value".to_vec(),
            },
        ),
        (
            native::delete(&cap, key, &hash).unwrap(),
            wire::Request::AppDelete {
                write_cap: cap.to_vec(),
                key: key.as_bytes().to_vec(),
                expect: hash.to_vec(),
            },
        ),
    ];
    for (encoded, expected) in cases {
        assert_eq!(decoder::decode_req(&encoded).unwrap(), expected);
    }
}
