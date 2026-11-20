use axum::{
    http::{HeaderMap, HeaderValue},
    response::Response,
};
use std::collections::BTreeMap;
pub fn from_request(headers: &HeaderMap) -> Result<BTreeMap<String, String>, ()> {
    let mut out = BTreeMap::new();
    for (key, value) in headers {
        let value = value.to_str().map_err(|_| ())?;
        out.insert(key.as_str().into(), value.into());
    }
    Ok(out)
}
fn hop_by_hop(key: &str, connection: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "transfer-encoding"
            | "upgrade"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "proxy-connection"
            | "te"
            | "trailer"
    ) || connection
        .split(',')
        .any(|name| name.trim().eq_ignore_ascii_case(key))
}
pub fn into_response(r: skrynia_tools::wire::v2compat::Response) -> Result<Response, ()> {
    let mut builder = Response::builder().status(r.status);
    let connection = r
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("connection"))
        .map(|(_, value)| value.as_str())
        .unwrap_or("");
    for (key, value) in &r.headers {
        if hop_by_hop(key, connection) {
            continue;
        }
        let value = HeaderValue::from_str(value).map_err(|_| ())?;
        builder = builder.header(key.as_str(), value);
    }
    builder.body(axum::body::Body::from(r.body)).map_err(|_| ())
}
