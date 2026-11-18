use super::{Grant, scope};
use std::collections::{HashMap, HashSet};
pub fn validate(grants: &HashMap<String, Grant>) -> Result<(), &'static str> {
    if grants.is_empty() {
        return Err("missing_root");
    }
    let roots = grants.values().filter(|g| g.parent.is_none()).count();
    if roots != 1 {
        return Err("invalid_roots");
    }
    let mut tokens = HashSet::new();
    for g in grants.values() {
        if !scope::valid_scope(&g.locations) {
            return Err("invalid_scope");
        }
        for user in &g.users {
            if !tokens.insert(user.hash.as_str()) {
                return Err("duplicate_user_token");
            }
        }
        if let Some(parent) = &g.parent {
            let p = grants.get(parent).ok_or("orphan_grant")?;
            if !scope::subset(&g.locations, &p.locations) {
                return Err("widened_scope");
            }
            if g.operations.iter().any(|op| !p.operations.contains(op)) {
                return Err("widened_operations");
            }
        }
    }
    let mut finished = HashSet::new();
    for g in grants.values() {
        let mut cursor = Some(g.id.as_str());
        let mut seen = HashSet::new();
        while let Some(id) = cursor {
            if finished.contains(id) {
                break;
            }
            if !seen.insert(id) {
                return Err("grant_cycle");
            }
            cursor = grants.get(id).ok_or("orphan_grant")?.parent.as_deref();
        }
        finished.extend(seen);
    }
    Ok(())
}
