use super::{cases, fixture::Server, guest};
use sha2::{Digest, Sha256};

#[test]
#[ignore = "requires separately built root WASM and real v3 kernel"]
fn missing_wrong_and_old_wire_tokens_are_denied_without_output() {
    let s = Server::start(Some(&guest::echo()));
    let leaked_verifier: [u8; 32] = Sha256::digest([0x42u8; 32]).into();
    for bad in [
        b"".as_slice(),
        &[1u8][..],
        &[1u8; 32][..],
        &[0u8; 33][..],
        &cases::authenticated(b"private", &[0x43u8; 32]),
        &cases::authenticated(b"private", &leaked_verifier),
    ] {
        let response = cases::post(&s.url, bad);
        assert_eq!(response.status(), 500);
        assert!(response.bytes().unwrap().is_empty());
    }
    let value = b"\0api\0request\xff";
    let good = cases::post(&s.url, &cases::authenticated(value, &[0x42; 32]));
    assert_eq!(good.status(), 200);
    assert_eq!(good.bytes().unwrap().as_ref(), value);
}

#[test]
#[ignore = "requires separately built root WASM and real v3 kernel"]
fn verifier_is_fail_closed_and_token_rotation_takes_effect_immediately() {
    let s = Server::start(Some(&guest::echo()));
    let message = b"operation";
    let old = cases::authenticated(message, &[0x42; 32]);
    assert_eq!(cases::post(&s.url, &old).status(), 200);
    s.corrupt(&[0x22; 31]);
    assert_eq!(cases::post(&s.url, &old).status(), 500);
    s.corrupt(&[0x22; 33]);
    assert_eq!(cases::post(&s.url, &old).status(), 500);
    s.rotate(&[0x88; 32]);
    assert_eq!(cases::post(&s.url, &old).status(), 500);
    let new = cases::authenticated(message, &[0x88; 32]);
    assert_eq!(cases::post(&s.url, &new).status(), 200);
    s.invalidate();
    assert_eq!(cases::post(&s.url, &new).status(), 500);
}
