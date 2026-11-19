use super::*;
use serde_json::json;
use std::collections::HashMap;
pub fn run<S: StateStore, R: RandomSource>(
    backend: &DurableBackend<S, R>,
    id: &str,
    token: &str,
) -> CompatResult<Value> {
    let state = backend.checked()?;
    let auth = DurableBackend::<S, R>::authority(&state)?;
    let grant = auth.user(token)?;
    if grant.id != id {
        return Err(CompatError::new(403, "invalid_grant_user_token"));
    }
    let mut value =
        serde_json::to_value(grant).map_err(|_| CompatError::new(500, "invalid_state"))?;
    let map = value
        .as_object_mut()
        .ok_or_else(|| CompatError::new(500, "invalid_state"))?;
    for name in ["issuerHash", "rootIssuerToken", "users"] {
        map.remove(name);
    }
    map.insert("userCount".into(), json!(grant.users.len()));
    let sizes: HashMap<String, u64> = state
        .objects
        .iter()
        .map(|(key, bytes)| (key.clone(), bytes.len() as u64))
        .collect();
    let usage = auth
        .usage(id, &sizes, &[])
        .ok_or_else(|| CompatError::new(500, "invalid_state"))?;
    map.insert(
        "usage".into(),
        json!({"bytes":usage.bytes,"objects":usage.objects}),
    );
    Ok(value)
}
