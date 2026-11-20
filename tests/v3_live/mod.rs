mod appdata;
mod guest;
mod process;
mod versions;
mod wire;
use skrynia_native_service::auth;
use skrynia_native_service::wire::{AppPrecond, Request};
#[test]
#[ignore = "requires a separately built Skrynia v3 binary and Program 0 WASM"]
fn external_user_program0_register_set_restart_revoke() {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let wasm = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").expect("SKRYNIA_V3_PROGRAM0_WASM");
    let mut server = process::Server::new(bin, wasm).unwrap();
    let admin = vec![5u8; 32];
    let iid = wire::call(
        &server.url,
        Request::Register {
            operator_cap: vec![85; 32],
            admin_cap: admin.clone(),
            code: guest::byte(65),
        },
    )
    .unwrap();
    assert_eq!(iid, auth::read_cap_vec(&admin));
    assert_eq!(wire::invoke(&server.url, &iid).unwrap(), b"A");
    let h_a = auth::hash_bytes(&guest::byte(65)).to_vec();
    wire::call(
        &server.url,
        Request::Set {
            admin_cap: admin.clone(),
            expect: AppPrecond::Present(h_a),
            code: guest::byte(66),
        },
    )
    .unwrap();
    assert_eq!(wire::invoke(&server.url, &iid).unwrap(), b"B");
    server.restart().unwrap();
    assert_eq!(wire::invoke(&server.url, &iid).unwrap(), b"B");
    let h_b = auth::hash_bytes(&guest::byte(66)).to_vec();
    wire::call(
        &server.url,
        Request::Revoke {
            admin_cap: admin,
            expect: AppPrecond::Present(h_b),
        },
    )
    .unwrap();
    assert!(wire::invoke(&server.url, &iid).is_err());
}
