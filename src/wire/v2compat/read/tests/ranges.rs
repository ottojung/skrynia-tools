use super::{fake, handle, request};
#[test]
fn ranges_and_suffixes() {
    let backend = fake();
    for (header, status, body, range) in [
        ("bytes=1-3", 206, "bcd", "bytes 1-3/6"),
        ("bytes=-2", 206, "ef", "bytes 4-5/6"),
        ("bytes=4-", 206, "ef", "bytes 4-5/6"),
        ("bytes=0-99", 206, "abcdef", "bytes 0-5/6"),
        ("bytes=10-12", 416, "", "bytes */6"),
        ("bytes=0-1,3-4", 200, "abcdef", ""),
        ("bytes=-0", 416, "", "bytes */6"),
    ] {
        let mut req = request("GET");
        req.headers.insert("Range".into(), header.into());
        let res = handle(&backend, &req);
        assert_eq!(res.status, status, "{header}");
        assert_eq!(res.body, body.as_bytes(), "{header}");
        if range.is_empty() {
            assert!(!res.headers.contains_key("content-range"), "{header}");
        } else {
            assert_eq!(res.headers.get("content-range").unwrap(), range);
        }
    }
}
#[test]
fn head_range_has_metadata_no_bytes() {
    let mut req = request("HEAD");
    req.headers.insert("range".into(), "bytes=1-2".into());
    let res = handle(&fake(), &req);
    assert_eq!(res.status, 206);
    assert_eq!(res.headers.get("content-length").unwrap(), "2");
    assert_eq!(res.headers.get("content-range").unwrap(), "bytes 1-2/6");
    assert!(res.body.is_empty());
}
