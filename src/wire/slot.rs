pub fn valid_prefix(prefix: &str) -> bool {
    valid_key(prefix)
}
pub fn key(prefix: &str, position: u64) -> String {
    format!("{prefix}.e000001.{position:012}")
}
pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 255
        && key.split('/').all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(i, c)| {
                    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || (i > 0 && c == b'.')
                })
        })
}
pub fn valid_matcher(matcher: &str) -> bool {
    if matcher.is_empty() || matcher == "*" {
        return true;
    }
    if let Some(prefix) = matcher.strip_suffix('*') {
        valid_key(prefix.strip_suffix('/').unwrap_or(prefix))
    } else {
        valid_key(matcher)
    }
}
