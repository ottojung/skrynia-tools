use super::req::{get, put};
use super::util::{call, setup};
use skrynia_native_service::auth;
use skrynia_native_service::wire::{AppPrecond, STATUS_ERR, STATUS_OK};

#[test]
fn per_object_cas_enforced_for_put() {
    let (r, d, l) = setup();
    let wc = [3u8; 32];
    let rc = auth::read_cap_vec(&wc);
    let h1 = call(&r, &d, &l, put(&wc, b"o", AppPrecond::Absent, b"a")).body;
    assert_eq!(
        call(&r, &d, &l, put(&wc, b"o", AppPrecond::Absent, b"b")).status,
        STATUS_ERR
    );
    let wrong = put(&wc, b"o", AppPrecond::Present(vec![0u8; 32]), b"b");
    assert_eq!(call(&r, &d, &l, wrong).status, STATUS_ERR);
    let ok = call(&r, &d, &l, put(&wc, b"o", AppPrecond::Present(h1), b"b"));
    assert_eq!(ok.status, STATUS_OK);
    assert_eq!(call(&r, &d, &l, get(&rc, b"o")).body, b"b");
}
