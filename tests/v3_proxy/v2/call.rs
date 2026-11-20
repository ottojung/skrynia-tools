use super::{cases, fixture::Server};
use skrynia_tools::wire::v2compat::{Request, Response};
pub(super) fn call(s: &Server, req: &Request) -> Response {
    let body = serde_json::to_vec(req).unwrap();
    let answer = cases::post(&s.url, &cases::authenticated(&body, &[0x42; 32]));
    assert_eq!(answer.status(), 200);
    serde_json::from_slice(&answer.bytes().unwrap()).unwrap()
}
pub(super) fn req(method: &str, path: &str) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: Default::default(),
        body: Vec::new(),
    }
}
