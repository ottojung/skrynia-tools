use super::{Grant, User};
use std::collections::HashMap;
pub struct Authority {
    pub(super) grants: HashMap<String, Grant>,
    pub(super) tokens: HashMap<String, String>,
}
impl Authority {
    pub fn from_grants(grants: impl IntoIterator<Item = Grant>) -> Result<Self, &'static str> {
        let mut result = Self {
            grants: HashMap::new(),
            tokens: HashMap::new(),
        };
        for grant in grants {
            if result.grants.contains_key(&grant.id) {
                return Err("duplicate_grant_id");
            }
            for User { hash, .. } in &grant.users {
                result.tokens.insert(hash.clone(), grant.id.clone());
            }
            result.grants.insert(grant.id.clone(), grant);
        }
        super::validate::validate(&result.grants)?;
        Ok(result)
    }
    pub fn grant(&self, id: &str) -> Option<&Grant> {
        self.grants.get(id)
    }
}
