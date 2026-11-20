use super::{cases::call_request, fixture::Server};
use skrynia_tools::wire::v2compat::Request;
#[test]
#[ignore = "requires a built generic kernel and v2-provider WASM"]
fn generic_v2_guest_rejects_mutation_and_preserves_ranges() {
    let server = Server::new();
    let mut req = Request {
        method: "GET".into(),
        path: "/store/public/file.txt".into(),
        query: None,
        headers: Default::default(),
        body: vec![],
    };
    req.headers.insert("range".into(), "bytes=2-4".into());
    let ranged = call_request(&server, &req);
    assert_eq!(ranged.status, 206);
    assert_eq!(ranged.body, b"cde");
    req.method = "PUT".into();
    req.headers.clear();
    req.body = b"unauthorized".to_vec();
    assert!(call_request(&server, &req).status >= 400);
    req.method = "POST".into();
    req.path = "/grants".into();
    assert_eq!(call_request(&server, &req).status, 501);
}
