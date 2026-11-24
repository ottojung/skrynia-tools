//! Pure parse of a captured request into an assertion-friendly shape.
use super::Seen;

pub fn parse(head: &str, body: Vec<u8>, sni: Option<String>) -> Seen {
    let mut lines = head.lines();
    let mut parts = lines.next().unwrap_or_default().split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default().to_owned();
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    let host = headers
        .iter()
        .find(|(name, _)| name == "host")
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    Seen {
        method,
        target,
        host,
        sni,
        headers,
        body,
    }
}
