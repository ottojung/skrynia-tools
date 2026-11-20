use super::fixture::Server;
use skrynia_tools::wire::v2compat::{Request, Response};
fn call(server: &Server, method: &str, path: &str) -> Response {
    let req = Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: Default::default(),
        body: Vec::new(),
    };
    call_request(server, &req)
}
pub(super) fn call_request(server: &Server, req: &Request) -> Response {
    let http = reqwest::blocking::Client::new()
        .post(&server.url)
        .body(serde_json::to_vec(&req).unwrap())
        .send()
        .unwrap();
    let status = http.status();
    let body = http.bytes().unwrap();
    assert_eq!(status, 200, "v3 transport failure body={:?}", body);
    serde_json::from_slice(&body)
        .unwrap_or_else(|e| panic!("invalid v2 reply: {e}; bytes={body:?}"))
}
#[test]
#[ignore = "requires a built generic kernel and v2-provider WASM"]
fn v2_get_is_evaluated_by_generic_v3_vm_not_by_kernel_get() {
    let server = Server::new();
    let client = reqwest::blocking::Client::new();
    assert_eq!(client.get(&server.url).send().unwrap().status(), 404);
    let get = call(&server, "GET", "/store/public/file.txt");
    assert_eq!(get.status, 200);
    assert_eq!(get.body, b"abcdef");
    let head = call(&server, "HEAD", "/store/public/file.txt");
    assert_eq!(head.status, 200);
    assert!(head.body.is_empty());
    assert_eq!(call(&server, "GET", "/store/../evil").status, 400);
}
