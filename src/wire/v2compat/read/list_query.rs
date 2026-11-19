use std::collections::BTreeMap;

fn decode_component(s: &str) -> Option<String> {
    let mut out = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hi = (b[i + 1] as char).to_digit(16)? as u8;
            let lo = (b[i + 2] as char).to_digit(16)? as u8;
            out.push((hi << 4) | lo);
            i += 3;
        } else if b[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}
pub fn params(query: Option<&str>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for pair in query.unwrap_or("").split('&').filter(|s| !s.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if let (Some(k), Some(v)) = (decode_component(k), decode_component(v)) {
            map.entry(k).or_insert(v);
        }
    }
    map
}
