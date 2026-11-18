use skrynia_tools::wire::v2compat::Response;
pub fn check(
    status: u16,
    body: &[u8],
    headers: &reqwest::header::HeaderMap,
    expected: &Response,
    case: &str,
) -> Result<(), String> {
    if status != expected.status || body != expected.body {
        return Err(format!(
            "{case}: expected {} {:?}, got {status} {:?}",
            expected.status, expected.body, body
        ));
    }
    for key in [
        "etag",
        "content-range",
        "content-length",
        "content-type",
        "cache-control",
        "accept-ranges",
    ] {
        let a = expected.headers.get(key).map(String::as_str);
        let b = headers.get(key).and_then(|v| v.to_str().ok());
        if a != b {
            return Err(format!(
                "{case}: status {status} / expected {}: {key}: expected {a:?}, got {b:?}",
                expected.status
            ));
        }
    }
    Ok(())
}
