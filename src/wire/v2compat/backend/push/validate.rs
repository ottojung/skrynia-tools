use super::{CompatError, SubReq, UpdateReq, scope};
const EVS: &[&str] = &["create", "replace", "delete"];

pub fn validate_sub(req: &SubReq) -> Result<(), CompatError> {
    if !scope::valid_scope(&req.locations) || !valid_events(&req.events.0) {
        return Err(bad());
    }
    validate_target(&req.endpoint, &req.keys.p256dh, &req.keys.auth)
}
pub fn validate_update(req: &UpdateReq) -> Result<(), CompatError> {
    validate_target(&req.endpoint, &req.keys.p256dh, &req.keys.auth)
}
fn valid_events(v: &[String]) -> bool {
    !v.is_empty()
        && v.iter().all(|e| EVS.contains(&e.as_str()))
        && (1..v.len()).all(|i| !v[..i].contains(&v[i]))
}
fn validate_target(ep: &str, p256dh: &str, auth: &str) -> Result<(), CompatError> {
    if !ep.starts_with("https://") || p256dh.is_empty() || auth.is_empty() {
        return Err(bad());
    }
    Ok(())
}
fn bad() -> CompatError {
    CompatError::new(400, "invalid_push_subscription")
}
