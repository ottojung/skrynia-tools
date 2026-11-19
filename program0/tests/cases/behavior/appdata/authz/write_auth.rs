use super::req::{del, get, put};
use super::util::{call, setup};
use skrynia_program0::auth;
use skrynia_program0::wire::{AppPrecond, STATUS_ERR};

#[test]
fn app_put_delete_require_write_cap() {
    let (r, d, l) = setup();
    let wc = [1u8; 32];
    let rc = auth::read_cap_vec(&wc);
    let h = call(&r, &d, &l, put(&wc, b"k", AppPrecond::Absent, b"v")).body;
    let bad_put = put(&rc, b"k", AppPrecond::Present(h.clone()), b"x");
    assert_eq!(call(&r, &d, &l, bad_put).status, STATUS_ERR);
    assert_eq!(call(&r, &d, &l, del(&rc, b"k", h)).status, STATUS_ERR);
    assert_eq!(call(&r, &d, &l, get(&rc, b"k")).body, b"v");
}
