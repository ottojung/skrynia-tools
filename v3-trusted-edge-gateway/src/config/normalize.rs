pub fn normalize_host(host: &str) -> Option<String> {
    let h = host.trim();
    if h.is_empty() || h.contains('/') || h.contains(' ') || h.contains('\\') {
        return None;
    }
    let lower = h.to_ascii_lowercase();
    if lower.contains('@') {
        return None;
    }
    Some(lower)
}

pub fn normalize_origin(origin: &str) -> Option<String> {
    let o = origin.trim();
    let (scheme, rest) = o.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "https" && scheme != "http" {
        return None;
    }
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('@')
    {
        return None;
    }
    Some(format!("{scheme}://{}", rest.to_ascii_lowercase()))
}
