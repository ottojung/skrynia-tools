mod token;
use super::{process, tail, wire};
use skrynia_program0::wire::{Request, V2Kind};

pub fn go() {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let wasm = std::env::var("SKRYNIA_V3_PROGRAM0_WASM").expect("SKRYNIA_V3_PROGRAM0_WASM");
    let mut server = process::Server::new(bin, wasm).unwrap();
    let token = token::private_root(&server);
    wire::ok(
        &server.url,
        req(token, V2Kind::Create, b"a.txt", None, b"v1"),
    )
    .unwrap();
    let bad = req(token, V2Kind::Replace, b"a.txt", Some([0u8; 32]), b"v2");
    assert_eq!(wire::err(&server.url, bad).unwrap(), "content_mismatch");
    tail::finish(&mut server, token);
}
pub(super) fn req(
    token: [u8; 32],
    kind: V2Kind,
    key: &[u8],
    expected: Option<[u8; 32]>,
    body: &[u8],
) -> Request {
    Request::V2Mutate {
        token,
        key: key.to_vec(),
        kind,
        expected,
        body: body.to_vec(),
    }
}
