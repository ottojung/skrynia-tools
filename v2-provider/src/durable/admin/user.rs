use super::{persist, user_quota};
use crate::bridge::Callback;
use crate::durable::{grants, lock, scan};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::{CompatError, grants::User};

fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn issue<I: Callback>(i: &I, id: &str, token: &str) -> Result<Vec<u8>, CompatError> {
    let guard = lock::acquire(i).ok_or_else(|| err(503, "unavailable"))?;
    let result = run(i, id, token);
    if lock::release(i, guard).is_err() {
        return Err(err(503, "mutation_outcome_unknown"));
    }
    result
}
fn run<I: Callback>(i: &I, id: &str, token: &str) -> Result<Vec<u8>, CompatError> {
    let auth = grants::load(i).map_err(|_| err(503, "unavailable"))?;
    let grant = auth.issuer(id, token)?;
    let sizes = scan::object_sizes(i).map_err(|_| err(503, "unavailable"))?;
    user_quota::check_user_meta(&auth, id, &sizes)?;
    let user_id = persist::hex(i, 16).map_err(|_| err(503, "unavailable"))?;
    let user_token = persist::hex(i, 32).map_err(|_| err(503, "unavailable"))?;
    let mut updated = grant.clone();
    updated.users.push(User {
        id: user_id.clone(),
        hash: hex::encode(Sha256::digest(user_token.as_bytes())),
    });
    persist::save(i, &updated).map_err(|_| err(503, "unavailable"))?;
    Ok(serde_json::json!({
        "grantId": id,
        "userId": user_id,
        "userToken": user_token
    })
    .to_string()
    .into_bytes())
}
