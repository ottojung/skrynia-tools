pub fn parse_http(resp: &[u8]) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let sep = resp
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or_else(|| {
            panic!(
                "http header end not found: received {} bytes, tail={:?}",
                resp.len(),
                String::from_utf8_lossy(&resp[resp.len().saturating_sub(128)..])
            )
        });
    let head = std::str::from_utf8(&resp[..sep]).expect("http header utf8");
    let mut lines = head.lines();
    let status = lines
        .next()
        .expect("status line")
        .split_whitespace()
        .nth(1)
        .expect("status code")
        .parse()
        .expect("numeric status");
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    let mut body = resp[sep + 4..].to_vec();
    let chunked = headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v.to_ascii_lowercase().contains("chunked"));
    if chunked {
        let raw_len = body.len();
        body = super::dechunk::dechunk(&body).unwrap_or_else(|e| {
            panic!(
                "incomplete chunked HTTP response: {e}; status={status}, raw_body_bytes={raw_len}, total_bytes={}",
                resp.len()
            )
        });
    } else if let Some((_, len)) = headers.iter().find(|(k, _)| k == "content-length") {
        let expected = len.parse::<usize>().expect("content-length");
        assert_eq!(
            body.len(),
            expected,
            "incomplete fixed-length HTTP response: status={status}, total_bytes={}",
            resp.len()
        );
    }
    (status, headers, body)
}
