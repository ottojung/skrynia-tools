mod paths;
use super::{
    backend::CompatBackend,
    reply,
    types::{Request, Response},
};

pub fn handle<B: CompatBackend>(backend: &B, req: &Request) -> Response {
    if req.method.eq_ignore_ascii_case("OPTIONS") {
        return preflight();
    }
    let method = req.method.to_ascii_uppercase();
    let path = req.path.as_str();
    match (method.as_str(), path) {
        ("GET", "/health") => with_cors(backend.health()),
        ("GET", "/version") => with_cors(backend.version()),
        ("GET", "/push/vapid") => with_cors(backend.push_vapid()),
        ("GET" | "HEAD", p) if p.starts_with("/store/") => {
            with_cors(super::read::handle(backend, &method, &p[7..], req))
        }
        ("POST" | "PUT" | "DELETE", p) if p.starts_with("/store/") => {
            with_cors(reply::error(501, "not_implemented"))
        }
        ("GET", "/store") => with_cors(reply::error(501, "not_implemented")),
        _ if paths::stub(method.as_str(), path) => with_cors(reply::error(501, "not_implemented")),
        _ if paths::known(path) => with_cors(reply::error(405, "method_not_allowed")),
        _ => with_cors(reply::error(404, "not_found")),
    }
}
fn with_cors(mut r: Response) -> Response {
    reply::cors(&mut r.headers);
    r
}
fn preflight() -> Response {
    let mut h = std::collections::BTreeMap::new();
    reply::cors(&mut h);
    h.insert(
        "access-control-allow-methods".into(),
        "GET, POST, PUT, DELETE, OPTIONS".into(),
    );
    h.insert("access-control-allow-headers".into(), "Authorization, Content-Type, X-Skrynia-Capability, X-Skrynia-If-SHA256, If-None-Match, If-Range, Range".into());
    h.insert("access-control-max-age".into(), "3600".into());
    Response {
        status: 204,
        headers: h,
        body: Vec::new(),
    }
}
