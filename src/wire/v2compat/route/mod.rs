mod http;
mod paths;
mod push;
pub(super) use super::backend::CompatBackend as Backend;
pub(super) use super::types::{Request as Req, Response as Resp};
use super::{
    backend::CompatBackend,
    reply,
    types::{Request, Response},
};
pub(super) use super::{reply as preply, types::Response as PResp};

pub fn handle<B: CompatBackend>(backend: &B, req: &Request) -> Response {
    if req.method.eq_ignore_ascii_case("OPTIONS") {
        return http::preflight();
    }
    let method = req.method.to_ascii_uppercase();
    let path = req.path.as_str();
    if let Some(r) = push::handle(backend, method.as_str(), path, req) {
        return http::with_cors(r);
    }
    let r = match (method.as_str(), path) {
        ("GET", "/health") => backend.health(),
        ("GET", "/version") => backend.version(),
        ("GET", "/push/vapid") => backend.push_vapid(),
        ("GET" | "HEAD", p) if p.starts_with("/store/") => {
            super::read::handle(backend, &method, &p[7..], req)
        }
        ("POST" | "PUT" | "DELETE", p) if p.starts_with("/store/") => {
            super::mutate::handle(backend, &method, &p[7..], req)
        }
        _ if paths::stub(method.as_str(), path) => reply::error(501, "not_implemented"),
        _ if paths::known(path) => reply::error(405, "method_not_allowed"),
        _ => reply::error(404, "not_found"),
    };
    http::with_cors(r)
}
