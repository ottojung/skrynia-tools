use super::setup::{hints, wake_setup};
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, read_cap};
#[test]
fn disjoint_namespaces_wake_shared_key() {
    let (d, wa, wb, ga, gb) = wake_setup();
    let wk = b"skrynia-wake-v1";
    ops::put_wake(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&wa),
        b"traveller-tes-v1",
        &AppPrecond::Absent,
        b"a",
        &ga,
        wk,
    )
    .unwrap();
    ops::put_wake(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&wb),
        b"traveller-vitalik-v1",
        &AppPrecond::Absent,
        b"b",
        &gb,
        wk,
    )
    .unwrap();
    let wn = read_cap(&[1u8; 32]);
    assert!(hints(&d, &wn, wk) >= 2);
    assert_eq!(hints(&d, &read_cap(&wa), wk), 0);
}
