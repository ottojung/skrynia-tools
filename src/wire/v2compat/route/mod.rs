mod admin;
mod cors;
mod paths;
pub(super) use super::types::Response as WireResponse;
use super::{
    backend::CompatBackend,
    reply,
    types::{Request, Response},
};

pub fn handle<B: CompatBackend>(backend: &B, req: &Request) -> Response {
    if req.method.eq_ignore_ascii_case("OPTIONS") {
        return cors::preflight();
    }
    let method = req.method.to_ascii_uppercase();
    let path = req.path.as_str();
    cors::with(match (method.as_str(), path) {
        ("GET", "/health") => backend.health(),
        ("GET", "/version") => backend.version(),
        ("GET", "/push/vapid") => backend.push_vapid(),
        ("GET" | "HEAD", p) if p.starts_with("/store/") => {
            super::read::handle(backend, method.as_str(), &p[7..], req)
        }
        ("POST" | "PUT" | "DELETE", p) if p.starts_with("/store/") => {
            super::mutate::handle(backend, method.as_str(), &p[7..], req)
        }
        ("GET", "/store") => reply::error(501, "not_implemented"),
        _ if path == "/grants" || path.starts_with("/grants/") => {
            admin::handle(backend, method.as_str(), path, req)
        }
        _ if paths::stub(method.as_str(), path) => reply::error(501, "not_implemented"),
        _ if paths::known(path) => reply::error(405, "method_not_allowed"),
        _ => reply::error(404, "not_found"),
    })
}
