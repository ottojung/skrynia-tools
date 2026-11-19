use super::{PResp, preply};
use std::collections::BTreeMap;
pub fn with_cors(mut r: PResp) -> PResp {
    preply::cors(&mut r.headers);
    r
}
pub fn preflight() -> PResp {
    let mut h = BTreeMap::new();
    preply::cors(&mut h);
    h.insert(
        "access-control-allow-methods".into(),
        "GET, POST, PUT, DELETE, OPTIONS".into(),
    );
    h.insert("access-control-allow-headers".into(), "Authorization, Content-Type, X-Skrynia-Capability, X-Skrynia-If-SHA256, If-None-Match, If-Range, Range".into());
    h.insert("access-control-max-age".into(), "3600".into());
    PResp {
        status: 204,
        headers: h,
        body: Vec::new(),
    }
}
