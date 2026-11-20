use super::{create_quota, persist};
use crate::bridge::Callback;
use crate::durable::{grants, lock, scan};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::{CompatError, grants::Grant};

fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn create<I: Callback>(i: &I, token: &str, body: &[u8]) -> Result<Vec<u8>, CompatError> {
    let guard = lock::acquire(i).ok_or_else(|| err(503, "unavailable"))?;
    let result = run(i, token, body);
    if lock::release(i, guard).is_err() {
        return Err(err(503, "mutation_outcome_unknown"));
    }
    result
}
fn run<I: Callback>(i: &I, token: &str, body: &[u8]) -> Result<Vec<u8>, CompatError> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| err(400, "invalid_grant_spec"))?;
    let list = grants::list(i).map_err(|_| err(503, "unavailable"))?;
    let auth = skrynia_tools::wire::v2compat::grants::Authority::from_grants(list)
        .map_err(|_| err(503, "unavailable"))?;
    let spec = auth.child_spec(token, &value)?;
    let parent = auth.user(token)?;
    if parent.max_bytes.is_some_and(|l| spec.max_bytes > l)
        || parent.max_objects.is_some_and(|l| spec.max_objects > l)
    {
        return Err(err(403, "grant_quota_exceeded"));
    }
    let sizes = scan::object_sizes(i).map_err(|_| err(503, "unavailable"))?;
    create_quota::check_parent_meta(&auth, &parent.id, &sizes)?;
    let id = persist::hex(i, 16).map_err(|_| err(503, "unavailable"))?;
    let issuer = persist::hex(i, 32).map_err(|_| err(503, "unavailable"))?;
    let grant = Grant {
        id: id.clone(),
        parent: Some(parent.id.clone()),
        locations: spec.locations,
        operations: spec.operations,
        max_bytes: Some(spec.max_bytes),
        max_objects: Some(spec.max_objects),
        issuer_hash: hex::encode(Sha256::digest(issuer.as_bytes())),
        root_issuer_token: None,
        users: Vec::new(),
    };
    persist::save(i, &grant).map_err(|_| err(503, "unavailable"))?;
    Ok(serde_json::json!({"id": id, "issuerToken": issuer})
        .to_string()
        .into_bytes())
}
