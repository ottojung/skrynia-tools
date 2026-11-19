use super::util::{put, server};
use crate::{guest, wire};
use skrynia_program0::auth;
use skrynia_program0::wire::{AppPrecond, Request};

#[test]
#[ignore = "requires a separately built Skrynia v3 binary and Program 0 WASM"]
fn external_user_program0_app_data_and_guest_callback_isolation() {
    let s = server();
    let wc = vec![7u8; 32];
    let rc = auth::read_cap_vec(&wc);
    put(&s.url, &wc, AppPrecond::Absent, b"v");
    let got = wire::call(
        &s.url,
        Request::AppGet {
            read_cap: rc.clone(),
            key: b"k".to_vec(),
        },
    )
    .unwrap();
    assert_eq!(got, b"v");
    let bad = Request::AppPut {
        write_cap: rc.clone(),
        key: b"k".to_vec(),
        expect: AppPrecond::Present(auth::hash_bytes(b"v").to_vec()),
        value: b"evil".to_vec(),
    };
    assert!(wire::call(&s.url, bad).is_err());
    let admin = vec![42u8; 32];
    let iid = wire::call(
        &s.url,
        Request::Register {
            admin_cap: admin.clone(),
            code: guest::callback_isolation_probe(),
        },
    )
    .unwrap();
    assert_eq!(iid, auth::read_cap_vec(&admin));
    let out = wire::call(
        &s.url,
        Request::Invoke {
            invoke_id: iid,
            input: vec![],
        },
    )
    .unwrap();
    assert_eq!(out, vec![1]);
}
