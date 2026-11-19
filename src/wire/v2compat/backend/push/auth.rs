use super::{CompatError, Record};
use sha2::{Digest, Sha256};
pub fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn valid(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn capability(rec: &Record, token: &str) -> Result<(), CompatError> {
    if !valid(token) || hash(token) != rec.cap_hash {
        return Err(CompatError::new(403, "invalid_subscription_capability"));
    }
    Ok(())
}
pub fn bearer(req: &super::Request) -> Result<&str, CompatError> {
    let v = req
        .headers
        .get("authorization")
        .map(String::as_str)
        .unwrap_or("");
    let t = v.strip_prefix("Bearer ").unwrap_or("");
    if t.len() == 64 && t.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(t)
    } else {
        Err(CompatError::new(401, "grant_user_token_required"))
    }
}
pub fn cap_header(req: &super::Request) -> Result<&str, CompatError> {
    let t = req
        .headers
        .get("x-skrynia-capability")
        .map(String::as_str)
        .unwrap_or("");
    if valid(t) {
        Ok(t)
    } else {
        Err(CompatError::new(403, "invalid_subscription_capability"))
    }
}
