//! Native P1 cross-namespace wake-grant protocol and negative security gates.
mod common;
mod crossns_cases;

use common::{WAKE, caps, data_exists, ns, seeded};
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;

/// A Wake grant is bound to (exact wake key, source namespace) and is not a
/// sender capability, a data write capability, or a grant for another key.
#[test]
fn issue5z_wake_grant_is_wake_only() {
    let d = seeded();
    const WG: [u8; 32] = [31u8; 32];
    let issue = |key: &[u8], grant: &[u8]| {
        push::issue_grant::<common::Mem, NoLock>(
            &d,
            &NoLock,
            "d",
            &push::Kind::Wake,
            &caps().0,
            grant,
            key,
            &ns(),
        )
    };
    assert!(issue(WAKE, &WG).is_ok());
    assert!(issue(b"other-v1", &[32u8; 32]).is_err());
    assert!(push::hint_list(&d, &NoLock, "d", &WG, &ns(), WAKE).is_err());
    let r = ops::put_wake(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&[9u8; 32]),
        b"traveller-vitalik-v1",
        &AppPrecond::Absent,
        b"v",
        &WG,
        WAKE,
    );
    assert!(r.is_err());
    assert!(!data_exists(&d, b"traveller-vitalik-v1"));
}
