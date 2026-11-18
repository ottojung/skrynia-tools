mod select;
use super::{
    super::types::{Request, Response},
    mime,
};
use std::collections::BTreeMap;

pub fn respond(method: &str, key: &str, bytes: &[u8], tag: &str, req: &Request) -> Response {
    let size = bytes.len() as u64;
    let sel = select::selection(req, tag, size);
    let mut h = BTreeMap::new();
    h.insert("content-type".into(), mime::for_key(key).into());
    h.insert("etag".into(), tag.into());
    h.insert("cache-control".into(), "no-cache".into());
    h.insert("accept-ranges".into(), "bytes".into());
    h.insert("x-content-type-options".into(), "nosniff".into());
    let body = match sel.status {
        416 => {
            h.insert("content-range".into(), format!("bytes */{size}"));
            Vec::new()
        }
        304 => Vec::new(),
        206 => {
            h.insert(
                "content-range".into(),
                format!("bytes {}-{}/{size}", sel.start, sel.end),
            );
            let len = if size == 0 {
                0
            } else {
                sel.end - sel.start + 1
            };
            h.insert("content-length".into(), len.to_string());
            if method == "HEAD" || len == 0 {
                Vec::new()
            } else {
                bytes[sel.start as usize..=sel.end as usize].to_vec()
            }
        }
        _ => {
            let len = if size == 0 {
                0
            } else {
                sel.end - sel.start + 1
            };
            h.insert("content-length".into(), len.to_string());
            if method == "HEAD" || len == 0 {
                Vec::new()
            } else {
                bytes.to_vec()
            }
        }
    };
    Response {
        status: sel.status,
        headers: h,
        body,
    }
}
