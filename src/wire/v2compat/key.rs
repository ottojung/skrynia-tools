pub fn decode(value: &str) -> Result<String, &'static str> {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && (i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit())
        {
            return Err("invalid_percent_encoding");
        }
        i += 1;
    }
    let decoded = percent_decode(value)?;
    if decoded.len() > 255 || !valid_key(&decoded) {
        return Err("invalid_key");
    }
    Ok(decoded)
}
fn percent_decode(s: &str) -> Result<String, &'static str> {
    let mut out = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hi = (b[i + 1] as char).to_digit(16).unwrap() as u8;
            let lo = (b[i + 2] as char).to_digit(16).unwrap() as u8;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "invalid_percent_encoding")
}
pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.split('/').all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(i, c)| {
                    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || (i > 0 && c == b'.')
                })
        })
}
