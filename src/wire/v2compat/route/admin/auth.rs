use super::{Request, Response, route_reply};
pub fn bearer(req: &Request) -> Result<&str, Response> {
    let value = req
        .headers
        .get("authorization")
        .or_else(|| req.headers.get("Authorization"))
        .and_then(|x| x.strip_prefix("Bearer "))
        .unwrap_or("");
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(route_reply::error(401, "grant_user_token_required"));
    }
    Ok(value)
}
pub fn issuer(req: &Request) -> Result<&str, Response> {
    let value = req
        .headers
        .get("authorization")
        .or_else(|| req.headers.get("Authorization"))
        .and_then(|x| x.strip_prefix("Bearer "))
        .unwrap_or("");
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(route_reply::error(403, "invalid_grant_issuer_token"));
    }
    Ok(value)
}
