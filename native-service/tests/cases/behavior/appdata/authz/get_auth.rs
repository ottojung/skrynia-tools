use super::req::{get, put};
use super::util::{call, setup};
use skrynia_native_service::auth;
use skrynia_native_service::wire::{AppPrecond, STATUS_ERR, STATUS_OK};

#[test]
fn app_get_requires_matching_read_cap() {
    let (r, d, l) = setup();
    let wc = [1u8; 32];
    let rc = auth::read_cap_vec(&wc);
    let p = call(&r, &d, &l, put(&wc, b"k", AppPrecond::Absent, b"v"));
    assert_eq!(p.status, STATUS_OK);
    assert_eq!(call(&r, &d, &l, get(&rc, b"k")).body, b"v");
    for bad in [[2u8; 32].to_vec(), wc.to_vec()] {
        assert_eq!(call(&r, &d, &l, get(&bad, b"k")).status, STATUS_ERR);
    }
}
