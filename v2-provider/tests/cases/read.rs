use super::{Fake, req};
use skrynia_v2_provider::dispatch;
#[test]
fn ordinary_guest_preserves_v2_get_head_and_range_semantics() {
    let mut request = req("GET", "/store/public/file.txt");
    let full = dispatch(&request, &Fake);
    assert_eq!(full.status, 200);
    assert_eq!(full.body, b"abcdef");
    assert!(full.headers.contains_key("etag"));
    request.method = "HEAD".into();
    let head = dispatch(&request, &Fake);
    assert_eq!(head.status, 200);
    assert!(head.body.is_empty());
    request.method = "GET".into();
    request.headers.insert("range".into(), "bytes=2-3".into());
    let part = dispatch(&request, &Fake);
    assert_eq!(part.status, 206);
    assert_eq!(part.body, b"cd");
    assert_eq!(
        part.headers.get("content-range").map(String::as_str),
        Some("bytes 2-3/6")
    );
}
#[test]
fn missing_and_invalid_paths_remain_v2_errors() {
    assert_eq!(
        dispatch(&req("GET", "/store/public/missing.txt"), &Fake).status,
        404
    );
    assert_eq!(dispatch(&req("GET", "/store/../evil"), &Fake).status, 400);
}

#[test]
fn preflight_is_legacy_guest_semantics_not_kernel_http() {
    let options = dispatch(&req("OPTIONS", "/store/public/file.txt"), &Fake);
    assert_eq!(options.status, 204);
    assert!(options.body.is_empty());
    assert_eq!(
        options
            .headers
            .get("access-control-allow-origin")
            .map(String::as_str),
        Some("*")
    );
}
