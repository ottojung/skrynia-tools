use super::{CompatError, CompatResult, DurableBackend, ListPage, RandomSource, StateStore, scope};

fn token_ok(token: &str) -> bool {
    token.len() == 64
        && token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn list_objects<S: StateStore, R: RandomSource>(
    backend: &DurableBackend<S, R>,
    token: &str,
    locations: &str,
    after: Option<&str>,
    limit: usize,
) -> CompatResult<ListPage> {
    if !token_ok(token) {
        return Err(CompatError::new(401, "grant_user_token_required"));
    }
    if !scope::valid_scope(locations) || locations.is_empty() {
        return Err(CompatError::new(400, "invalid_locations"));
    }
    let limit = limit.clamp(1, 1000);
    let state = backend.checked()?;
    let authority = DurableBackend::<S, R>::authority(&state)?;
    let grant = authority.user(token)?;
    if !grant.operations.iter().any(|op| op == "list") {
        return Err(CompatError::new(403, "grant_operation_forbidden"));
    }
    if !scope::subset(locations, &grant.locations) {
        return Err(CompatError::new(403, "location_not_authorized"));
    }
    let mut keys = Vec::new();
    for key in state.objects.keys() {
        if after.is_some_and(|a| key.as_str() <= a) {
            continue;
        }
        if scope::matches(locations, key) {
            keys.push(key.clone());
            if keys.len() > limit {
                break;
            }
        }
    }
    let next = if keys.len() > limit {
        keys.truncate(limit);
        keys.last().cloned()
    } else {
        None
    };
    Ok(ListPage { keys, next })
}
