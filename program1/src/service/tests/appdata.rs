use super::support::{Mem, err, ok, open_p};
use skrynia_native_service::auth;
use skrynia_native_service::wire::{AppPrecond, Request};

#[test]
fn appdata_read_write_caps_separated() {
    let d = Mem::default();
    let reg = open_p(&d);
    let wc = vec![9u8; 32];
    let rc = auth::read_cap_vec(&wc);
    ok(
        &reg,
        &d,
        Request::AppPut {
            write_cap: wc.clone(),
            key: b"k".to_vec(),
            expect: AppPrecond::Absent,
            value: b"v".to_vec(),
        },
    );
    let got = ok(
        &reg,
        &d,
        Request::AppGet {
            read_cap: rc.clone(),
            key: b"k".to_vec(),
        },
    );
    assert_eq!(got, b"v");
    let h = auth::hash_bytes(b"v").to_vec();
    let _ = err(
        &reg,
        &d,
        Request::AppPut {
            write_cap: rc,
            key: b"k".to_vec(),
            expect: AppPrecond::Present(h),
            value: b"x".to_vec(),
        },
    );
    let _ = err(
        &reg,
        &d,
        Request::AppGet {
            read_cap: wc,
            key: b"k".to_vec(),
        },
    );
}
