use super::persist;
use crate::bridge::Callback;
use crate::durable::{grants, lock};
use skrynia_tools::wire::v2compat::CompatError;

fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn user<I: Callback>(i: &I, id: &str, user_id: &str, token: &str) -> Result<(), CompatError> {
    let guard = lock::acquire(i).ok_or_else(|| err(503, "unavailable"))?;
    let result = run(i, id, user_id, token);
    if lock::release(i, guard).is_err() {
        return Err(err(503, "mutation_outcome_unknown"));
    }
    result
}
fn run<I: Callback>(i: &I, id: &str, user_id: &str, token: &str) -> Result<(), CompatError> {
    let auth = grants::load(i).map_err(|_| err(503, "unavailable"))?;
    let grant = auth.issuer(id, token)?;
    let mut updated = grant.clone();
    let before = updated.users.len();
    updated.users.retain(|u| u.id != user_id);
    if updated.users.len() == before {
        return Err(err(404, "user_not_found"));
    }
    persist::save(i, &updated).map_err(|_| err(503, "unavailable"))
}
