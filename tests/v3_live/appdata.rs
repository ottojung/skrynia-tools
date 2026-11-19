use super::{guest, process, wire};
use skrynia_program0::wire::Request;

#[test]
#[ignore = "requires a separately built Skrynia v3 binary and Program 0 WASM"]
fn external_user_program0_app_data_and_guest_callback_isolation() {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let wasm = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").expect("SKRYNIA_V3_PROGRAM0_WASM");
    let server = process::Server::new(bin, wasm).unwrap();
    let app = vec![7u8; 32];
    wire::call(
        &server.url,
        Request::AppPut {
            app: app.clone(),
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        },
    )
    .unwrap();
    assert_eq!(
        wire::call(
            &server.url,
            Request::AppGet {
                app: app.clone(),
                key: b"k".to_vec()
            }
        )
        .unwrap(),
        b"v"
    );
    let id = wire::call(
        &server.url,
        Request::Register {
            code: guest::callback_isolation_probe(),
        },
    )
    .unwrap();
    let out = wire::call(&server.url, Request::Invoke { id, input: app }).unwrap();
    assert_eq!(out, vec![1]);
}
