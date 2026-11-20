use super::{cases::call_request, fixture::Server};
use skrynia_tools::wire::v2compat::Request;
use std::collections::BTreeMap;
const TOKEN: &str = "0000000000000000000000000000000000000000000000000000000000000000";
fn req(method: &str, body: Vec<u8>) -> Request {
    let mut h = BTreeMap::new();
    h.insert("authorization".into(), format!("Bearer {TOKEN}"));
    Request {
        method: method.into(),
        path: "/store/test/doc.txt".into(),
        query: None,
        headers: h,
        body,
    }
}
#[test]
#[ignore = "requires a built generic kernel and v2-provider WASM"]
fn v2_positive_create_replace_and_read_over_real_generic_callback() {
    let server = Server::new();
    assert_eq!(
        call_request(&server, &req("POST", b"alpha".to_vec())).status,
        201
    );
    let get = call_request(
        &server,
        &Request {
            method: "GET".into(),
            path: "/store/test/doc.txt".into(),
            query: None,
            headers: BTreeMap::new(),
            body: vec![],
        },
    );
    assert_eq!(get.status, 200);
    assert_eq!(get.body, b"alpha");
    let etag = get
        .headers
        .get("etag")
        .unwrap()
        .trim_matches('"')
        .to_string();
    let mut bad = req("PUT", b"beta".to_vec());
    bad.headers
        .insert("x-skrynia-if-sha256".into(), "0".repeat(64));
    assert_eq!(call_request(&server, &bad).status, 412);
    let mut good = req("PUT", b"beta".to_vec());
    good.headers.insert("x-skrynia-if-sha256".into(), etag);
    assert_eq!(call_request(&server, &good).status, 200);
    let get2 = call_request(
        &server,
        &Request {
            method: "GET".into(),
            path: "/store/test/doc.txt".into(),
            query: None,
            headers: BTreeMap::new(),
            body: vec![],
        },
    );
    assert_eq!(get2.body, b"beta");
}
