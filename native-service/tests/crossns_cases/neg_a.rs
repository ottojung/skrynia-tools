use super::hlp::pw;
use super::setup::{hints, iw, wake_setup};
use skrynia_native_service::read_cap;
#[test]
fn foreign_source_wrong_key_revoked_and_unauthorized() {
    let (d, wa, wb, ga, _) = wake_setup();
    let (wk, wn) = (b"skrynia-wake-v1".as_slice(), read_cap(&[1u8; 32]));
    assert!(pw(&d, &wb, b"k", b"v", &ga, wk).is_err());
    assert_eq!(hints(&d, &wn, wk), 0);
    assert!(pw(&d, &wa, b"k", b"v", &ga, b"wrong-key").is_err());
    assert!(pw(&d, &wa, b"k", b"v", &[9u8; 32], wk).is_err());
    iw(&d, &[1u8; 32], &ga, wk, &wb);
    assert!(pw(&d, &wa, b"k2", b"v", &ga, wk).is_err());
    assert_eq!(hints(&d, &wn, wk), 0);
}
