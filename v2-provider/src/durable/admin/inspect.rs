use crate::bridge::Callback;
use crate::durable::{grants, lock, scan};
use skrynia_tools::wire::v2compat::CompatError;

fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn get<I: Callback>(i: &I, id: &str, token: &str) -> Result<Vec<u8>, CompatError> {
    let guard = lock::acquire(i).ok_or_else(|| err(503, "unavailable"))?;
    let result = run(i, id, token);
    if lock::release(i, guard).is_err() {
        return Err(err(503, "mutation_outcome_unknown"));
    }
    result
}
fn run<I: Callback>(i: &I, id: &str, token: &str) -> Result<Vec<u8>, CompatError> {
    let auth = grants::load(i).map_err(|_| err(503, "unavailable"))?;
    let caller = auth.user(token)?;
    if caller.id != id {
        return Err(err(403, "grant_operation_forbidden"));
    }
    let grant = auth.grant(id).ok_or_else(|| err(404, "grant_not_found"))?;
    let sizes = scan::object_sizes(i).map_err(|_| err(503, "unavailable"))?;
    let usage = auth
        .usage(id, &sizes, &[])
        .ok_or_else(|| err(503, "unavailable"))?;
    Ok(serde_json::json!({
        "id": grant.id,
        "parent": grant.parent,
        "locations": grant.locations,
        "operations": grant.operations,
        "maxBytes": grant.max_bytes,
        "maxObjects": grant.max_objects,
        "bytesUsed": usage.bytes,
        "objectsUsed": usage.objects,
    })
    .to_string()
    .into_bytes())
}
