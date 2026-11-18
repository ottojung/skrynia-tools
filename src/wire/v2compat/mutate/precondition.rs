use super::{Request, Response, cache, reply};
pub fn check(req: &Request, method: &str) -> Result<Option<String>, Response> {
    for name in [
        "if-match",
        "if-none-match",
        "if-modified-since",
        "if-unmodified-since",
        "if-range",
    ] {
        if cache::header(&req.headers, name).is_some() {
            return Err(reply::error(400, "invalid_precondition"));
        }
    }
    let Some(value) = cache::header(&req.headers, "x-skrynia-if-sha256") else {
        return Ok(None);
    };
    if method == "POST"
        || value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(reply::error(400, "invalid_precondition"));
    }
    Ok(Some(value.into()))
}
