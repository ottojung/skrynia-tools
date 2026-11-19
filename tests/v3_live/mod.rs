mod guest;
mod process;
mod startup_diag;
mod wire;
use skrynia_program0::wire::Request;

#[test]
#[ignore = "requires a separately built Skrynia v3 binary and Program 0 WASM"]
fn external_user_program0_register_set_restart_revoke() {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let wasm = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").expect("SKRYNIA_V3_PROGRAM0_WASM");
    let mut server = process::Server::new(bin, wasm).unwrap();
    let id = wire::call(
        &server.url,
        Request::Register {
            code: guest::byte(65),
        },
    )
    .unwrap();
    assert_eq!(id.len(), 32);
    assert_eq!(wire::invoke(&server.url, &id).unwrap(), b"A");
    wire::call(
        &server.url,
        Request::Set {
            id: id.clone(),
            code: guest::byte(66),
        },
    )
    .unwrap();
    assert_eq!(wire::invoke(&server.url, &id).unwrap(), b"B");
    server.restart().unwrap();
    assert_eq!(wire::invoke(&server.url, &id).unwrap(), b"B");
    wire::call(&server.url, Request::Revoke { id: id.clone() }).unwrap();
    assert!(wire::invoke(&server.url, &id).is_err());
}
