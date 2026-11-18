use super::{
    backend::CompatBackend,
    reply,
    types::{Request, Response},
};

pub fn handle<B: CompatBackend>(backend: &B, req: &Request) -> Response {
    if req.method.eq_ignore_ascii_case("OPTIONS") {
        return preflight();
    }
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/health") => with_cors(backend.health()),
        ("GET", "/version") => with_cors(backend.version()),
        ("GET", "/push/vapid") => with_cors(backend.push_vapid()),
        _ => reply::json(404, br#"{"error":"not_found"}"#.to_vec()),
    }
}

fn with_cors(mut response: Response) -> Response {
    reply::cors(&mut response.headers);
    response
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
