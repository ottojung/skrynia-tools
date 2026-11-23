use super::hlp::pw;
use super::setup::wake_setup;
use skrynia_native_service::registry::appdata::push;
#[test]
fn crossns_quota_fails_closed() {
    let (d2, wa2, _, ga2, _) = wake_setup();
    let wk = b"shared-wake-v1";
    for i in 0..push::MAX_HINTS {
        pw(&d2, &wa2, format!("k{i:04}").as_bytes(), b"v", &ga2, wk).unwrap();
    }
    assert!(pw(&d2, &wa2, b"over", b"v", &ga2, wk).is_err());
}
