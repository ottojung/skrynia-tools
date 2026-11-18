use super::{fake, handle, request};
#[test]
fn weak_or_multiple_cache_validators_beat_range() {
    let backend = fake();
    let first = handle(&backend, &request("GET"));
    let tag = first.headers.get("etag").unwrap().clone();
    for value in [format!("W/{tag}"), format!("\"other\", {tag}"), "*".into()] {
        let mut req = request("GET");
        req.headers.insert("If-None-Match".into(), value);
        req.headers.insert("range".into(), "bytes=0-2".into());
        let res = handle(&backend, &req);
        assert_eq!(res.status, 304);
        assert!(res.body.is_empty());
    }
}
#[test]
fn if_range_requires_matching_strong_etag() {
    let backend = fake();
    let tag = handle(&backend, &request("GET")).headers["etag"].clone();
    for (value, status) in [
        ("wrong".to_string(), 200),
        (format!("W/{tag}"), 200),
        (tag, 206),
    ] {
        let mut req = request("GET");
        req.headers.insert("if-range".into(), value);
        req.headers.insert("range".into(), "bytes=0-1".into());
        assert_eq!(handle(&backend, &req).status, status);
    }
}
