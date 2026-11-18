use super::{Authority, CompatError, Grant, scope};
use sha2::{Digest, Sha256};
fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
impl Authority {
    pub fn user(&self, token: &str) -> Result<&Grant, CompatError> {
        self.tokens
            .get(&hash(token))
            .and_then(|id| self.grants.get(id))
            .ok_or_else(|| CompatError::new(403, "invalid_grant_user_token"))
    }
    pub fn issuer(&self, id: &str, token: &str) -> Result<&Grant, CompatError> {
        let grant = self
            .grants
            .get(id)
            .ok_or_else(|| CompatError::new(404, "grant_not_found"))?;
        if grant.issuer_hash != hash(token) {
            return Err(CompatError::new(403, "invalid_grant_issuer_token"));
        }
        Ok(grant)
    }
    pub fn grant_for(
        &self,
        token: &str,
        operation: &str,
        key: &str,
    ) -> Result<&Grant, CompatError> {
        let grant = self.user(token)?;
        if !grant.operations.iter().any(|op| op == operation) {
            return Err(CompatError::new(403, "grant_operation_forbidden"));
        }
        if !scope::matches(&grant.locations, key) {
            return Err(CompatError::new(403, "location_not_authorized"));
        }
        Ok(grant)
    }
}
