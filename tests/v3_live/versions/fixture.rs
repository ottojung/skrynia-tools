use crate::{guest, process, wire};
use skrynia_program0::wire::{AdminRequest, Request};
pub fn server() -> (process::Server, Vec<u8>, Vec<u8>) {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let wasm = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").expect("SKRYNIA_V3_PROGRAM0_WASM");
    let server = process::Server::new(bin, wasm).unwrap();
    let admin = vec![14; 32];
    let id = wire::call(
        &server.url,
        Request::Register {
            operator_cap: vec![85; 32],
            admin_cap: admin.clone(),
            code: guest::byte(65),
        },
    )
    .unwrap();
    (server, admin, id)
}
pub fn admin(url: &str, req: AdminRequest) -> Result<Vec<u8>, String> {
    wire::call(url, Request::Admin(req))
}
pub fn ready(url: &str, admin_cap: &[u8], code: Vec<u8>) -> Vec<u8> {
    let hash = admin(
        url,
        AdminRequest::StageCode {
            admin_cap: admin_cap.to_vec(),
            code,
        },
    )
    .unwrap();
    admin(
        url,
        AdminRequest::SealCode {
            admin_cap: admin_cap.to_vec(),
            hash: hash.clone(),
        },
    )
    .unwrap();
    hash
}
