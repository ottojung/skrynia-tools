mod backend;
mod checks;
use super::{Request, Response, Unavailable, backend as api_backend, handle};
use backend::Memory;
fn req(method: &str) -> Request {
    let mut r = Request {
        method: method.into(),
        path: "/store/a.txt".into(),
        query: None,
        headers: Default::default(),
        body: b"new".to_vec(),
    };
    r.headers
        .insert("authorization".into(), format!("Bearer {}", "a".repeat(64)));
    r
}
