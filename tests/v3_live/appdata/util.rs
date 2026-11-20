use crate::{process, wire};
use skrynia_native_service::wire::{AppPrecond, Request};

pub fn server() -> process::Server {
    let b = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let w = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").expect("SKRYNIA_V3_PROGRAM0_WASM");
    process::Server::new(b, w).unwrap()
}

pub fn put(url: &str, wc: &[u8], expect: AppPrecond, val: &[u8]) {
    wire::call(
        url,
        Request::AppPut {
            write_cap: wc.to_vec(),
            key: b"k".to_vec(),
            expect,
            value: val.to_vec(),
        },
    )
    .unwrap();
}
