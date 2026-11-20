use super::{WireResponse, reply};
pub fn with(mut r: WireResponse) -> WireResponse {
    reply::cors(&mut r.headers);
    r
}
pub fn preflight() -> WireResponse {
    let mut h = std::collections::BTreeMap::new();
    reply::cors(&mut h);
    h.insert(
        "access-control-allow-methods".into(),
        "GET, POST, PUT, DELETE, OPTIONS".into(),
    );
    h.insert(
        "access-control-allow-headers".into(),
        "Authorization, Content-Type, X-Skrynia-Capability, X-Skrynia-If-SHA256, If-None-Match, If-Range, Range".into(),
    );
    h.insert("access-control-max-age".into(), "3600".into());
    WireResponse {
        status: 204,
        headers: h,
        body: Vec::new(),
    }
}
