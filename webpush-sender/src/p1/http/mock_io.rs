//! Minimal HTTP/1.1 request reader for the disposable mock edge.
use std::io::Read;
use std::net::TcpStream;

#[derive(Clone)]
pub struct Seen {
    pub request_line: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub raw: Vec<u8>,
}

pub fn field<'a>(s: &'a Seen, key: &str) -> &'a str {
    s.headers
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .unwrap_or("")
}

pub fn read_request(stream: &mut TcpStream) -> Seen {
    let (mut raw, mut one) = (Vec::new(), [0u8; 1]);
    let end = loop {
        stream.read_exact(&mut one).unwrap();
        raw.push(one[0]);
        if raw.ends_with(b"\r\n\r\n") {
            break raw.len();
        }
    };
    let text = String::from_utf8_lossy(&raw[..end]);
    let mut lines = text.lines();
    let request_line = lines.next().unwrap().to_owned();
    let (mut headers, mut len) = (Vec::new(), 0usize);
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("content-length") {
                len = value.trim().parse().unwrap();
            }
            headers.push((key.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).unwrap();
    raw.extend_from_slice(&body);
    Seen {
        request_line,
        headers,
        body,
        raw,
    }
}
