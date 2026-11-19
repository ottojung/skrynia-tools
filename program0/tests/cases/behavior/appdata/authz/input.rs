use super::util::{call, setup};
use skrynia_program0::wire::{AppPrecond, Request, STATUS_ERR};

#[test]
fn invalid_caps_and_keys_are_rejected() {
    let (r, d, l) = setup();
    let wc = [4u8; 32];
    let bad = Request::AppGet {
        read_cap: vec![1, 2, 3],
        key: b"k".to_vec(),
    };
    assert_eq!(call(&r, &d, &l, bad).status, STATUS_ERR);
    let badk = Request::AppGet {
        read_cap: wc.to_vec(),
        key: vec![],
    };
    assert_eq!(call(&r, &d, &l, badk).status, STATUS_ERR);
    let badw = Request::AppPut {
        write_cap: vec![0u8; 31],
        key: b"k".to_vec(),
        expect: AppPrecond::Absent,
        value: b"v".to_vec(),
    };
    assert_eq!(call(&r, &d, &l, badw).status, STATUS_ERR);
    let badd = Request::AppDelete {
        write_cap: vec![0u8; 31],
        key: b"k".to_vec(),
        expect: vec![0u8; 32],
    };
    assert_eq!(call(&r, &d, &l, badd).status, STATUS_ERR);
}
