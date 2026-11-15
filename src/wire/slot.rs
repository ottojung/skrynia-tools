pub fn valid_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && !prefix.starts_with('.')
        && prefix
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
}
pub fn key(prefix: &str, position: u64) -> String {
    format!("{prefix}.e000001.{position:012}")
}

pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 256
        && !key.starts_with('.')
        && key
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
}
pub fn valid_matcher(matcher: &str) -> bool {
    if matcher.is_empty() || matcher == "*" {
        return true;
    }
    let value = matcher.strip_suffix('*').unwrap_or(matcher);
    valid_key(value)
}
