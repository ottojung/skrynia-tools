use super::req::{del, get, put};
use super::util::{call, setup};
use skrynia_program0::auth;
use skrynia_program0::wire::{AppPrecond, STATUS_ERR, STATUS_OK};

#[test]
fn per_object_cas_enforced_for_delete() {
    let (r, d, l) = setup();
    let wc = [3u8; 32];
    let rc = auth::read_cap_vec(&wc);
    let h = call(&r, &d, &l, put(&wc, b"o", AppPrecond::Absent, b"a")).body;
    assert_eq!(
        call(&r, &d, &l, del(&wc, b"o", vec![0u8; 32])).status,
        STATUS_ERR
    );
    assert_eq!(call(&r, &d, &l, del(&wc, b"o", h)).status, STATUS_OK);
    assert_eq!(call(&r, &d, &l, get(&rc, b"o")).status, STATUS_ERR);
}
