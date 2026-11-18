use super::{Request, Response, cache, reply};
pub fn get(req: &Request) -> Result<&str, Response> {
    let value = cache::header(&req.headers, "authorization")
        .and_then(|x| x.strip_prefix("Bearer "))
        .unwrap_or("");
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(reply::error(401, "grant_user_token_required"));
    }
    Ok(value)
}
