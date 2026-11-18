use super::{Response, select::Selection};
use std::collections::BTreeMap;
pub fn build(method: &str, key: &str, tag: &str, bytes: Vec<u8>, sel: Selection) -> Response {
    let size = bytes.len();
    let mut headers = BTreeMap::new();
    headers.insert("content-type".into(), super::mime::for_key(key).into());
    headers.insert("etag".into(), tag.into());
    headers.insert("cache-control".into(), "no-cache".into());
    headers.insert("accept-ranges".into(), "bytes".into());
    headers.insert("x-content-type-options".into(), "nosniff".into());
    if sel.status == 416 {
        headers.insert("content-range".into(), format!("bytes */{size}"));
    } else if sel.status != 304 {
        if sel.status == 206 {
            headers.insert(
                "content-range".into(),
                format!("bytes {}-{}/{size}", sel.start, sel.end),
            );
        }
        let len = if size == 0 {
            0
        } else {
            sel.end - sel.start + 1
        };
        headers.insert("content-length".into(), len.to_string());
    }
    let body = if method == "HEAD" || !matches!(sel.status, 200 | 206) || size == 0 {
        Vec::new()
    } else {
        bytes[sel.start..=sel.end].to_vec()
    };
    Response {
        status: sel.status,
        headers,
        body,
    }
}
