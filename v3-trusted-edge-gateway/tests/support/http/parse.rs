pub fn parse_http(resp: &[u8]) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let sep = resp
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("http header end");
    let head = std::str::from_utf8(&resp[..sep]).unwrap();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
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
        body = super::dechunk::dechunk(&body).expect("complete chunked HTTP response");
    } else if let Some((_, len)) = headers.iter().find(|(k, _)| k == "content-length") {
        assert_eq!(
            body.len(),
            len.parse::<usize>().expect("content-length"),
            "complete fixed-length HTTP response"
        );
    }
    (status, headers, body)
}
