use super::setup::{hints, wake_setup};
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, hash_bytes, read_cap};
#[test]
fn disjoint_delete_and_dynamic_thread_keys_wake() {
    let (d, wa, wb, ga, gb) = wake_setup();
    let wk = b"skrynia-wake-v1";
    ops::put_wake(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&wa),
        b"traveller-tes-v1",
        &AppPrecond::Absent,
        b"v",
        &ga,
        wk,
    )
    .unwrap();
    ops::delete_wake(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&wa),
        b"traveller-tes-v1",
        &hash_bytes(b"v"),
        &ga,
        wk,
    )
    .unwrap();
    for i in 0..113 {
        let k = format!("thread-{i:03}-tes-v1");
        ops::put_wake(
            &d,
            &NoLock,
            "d",
            WriteAuth::External(&wb),
            k.as_bytes(),
            &AppPrecond::Absent,
            b"m",
            &gb,
            wk,
        )
        .unwrap();
    }
    assert!(hints(&d, &read_cap(&[1u8; 32]), wk) >= 115);
}
