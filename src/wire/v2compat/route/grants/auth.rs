use super::{Req, Resp, preply};
pub fn token(req: &Req, issuer_only: bool) -> Result<&str, Resp> {
    let value = req
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("authorization"))
        .map(|(_, value)| value.as_str())
        .unwrap_or("");
    let raw = value.strip_prefix("Bearer ").unwrap_or("");
    if raw.len() == 64
        && raw
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(raw)
    } else if issuer_only {
        Ok("")
    } else {
        Err(preply::error(401, "grant_user_token_required"))
    }
}
