use super::Grant;
use std::collections::{HashMap, HashSet};
pub fn check(grants: &HashMap<String, Grant>) -> Result<(), &'static str> {
    let mut issuers = HashSet::new();
    let mut users = HashSet::new();
    for grant in grants.values() {
        if !issuers.insert(grant.issuer_hash.as_str()) {
            return Err("duplicate_issuer_token");
        }
        for user in &grant.users {
            if !users.insert(user.hash.as_str()) {
                return Err("duplicate_user_token");
            }
        }
    }
    if issuers.iter().any(|hash| users.contains(hash)) {
        return Err("issuer_user_token_collision");
    }
    Ok(())
}
