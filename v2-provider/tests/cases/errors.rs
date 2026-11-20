use super::{Fake, req};
use skrynia_v2_provider::{Callback, InvokeError, dispatch};
struct Offline;
impl Callback for Offline {
    fn invoke(&self, _: &[u8]) -> Result<Vec<u8>, InvokeError> {
        Err(InvokeError::Unavailable)
    }
}
#[test]
fn missing_capability_is_503_not_an_invented_404() {
    let response = dispatch(&req("GET", "/store/public/file.txt"), &Offline);
    assert_eq!(response.status, 503);
    assert!(String::from_utf8_lossy(&response.body).contains("unavailable"));
}
#[test]
fn unsupported_push_and_unauthenticated_grants_never_succeed() {
    assert!(dispatch(&req("POST", "/grants"), &Fake::default()).status >= 400);
    assert_eq!(
        dispatch(&req("POST", "/push/subscriptions"), &Fake::default()).status,
        501
    );
}
#[test]
fn unprivileged_mutation_has_no_successful_fallback() {
    let r = dispatch(&req("PUT", "/store/public/file.txt"), &Offline);
    assert!(r.status >= 400);
}
