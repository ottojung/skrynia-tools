use super::types::Response;
use std::collections::BTreeMap;

pub fn json(status: u16, body: Vec<u8>) -> Response {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".into(), "application/json".into());
    headers.insert("cache-control".into(), "no-store".into());
    cors(&mut headers);
    Response {
        status,
        headers,
        body,
    }
}

pub fn cors(headers: &mut BTreeMap<String, String>) {
    headers.insert("access-control-allow-origin".into(), "*".into());
    headers
        .entry("access-control-expose-headers".into())
        .or_insert_with(|| "ETag".into());
}
