use super::{Authority, CompatError, CompatResult, Request, State, bpush, scope};
use sha2::{Digest, Sha256};

pub fn bearer(req: &Request) -> Result<&str, CompatError> {
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
pub fn cap(req: &Request) -> Result<&str, CompatError> {
    let t = req
        .headers
        .get("x-skrynia-capability")
        .map(String::as_str)
        .unwrap_or("");
    if t.len() == 64 && t.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(t)
    } else {
        Err(CompatError::new(403, "invalid_subscription_capability"))
    }
}
pub fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn check_cap(rec: &bpush::Record, token: &str) -> CompatResult<()> {
    if hash(token) != rec.cap_hash {
        return Err(CompatError::new(403, "invalid_subscription_capability"));
    }
    Ok(())
}
pub fn subscribe_grant(state: &State, token: &str, locations: &str) -> CompatResult<String> {
    let auth = Authority::from_grants(state.grants.clone())
        .map_err(|e| CompatError::new(500, format!("grant_graph:{e}")))?;
    let grant = auth.user(token)?;
    if !grant.operations.iter().any(|op| op == "subscribe") {
        return Err(CompatError::new(403, "grant_operation_forbidden"));
    }
    if !scope::subset(locations, &grant.locations) {
        return Err(CompatError::new(403, "location_not_authorized"));
    }
    Ok(grant.id.clone())
}
