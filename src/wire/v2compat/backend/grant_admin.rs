use super::{CompatError, CompatResult};
use serde_json::Value;
pub trait GrantAdmin {
    fn grant_create(&self, _: &str, _: &Value) -> CompatResult<Value> {
        Err(CompatError::new(501, "not_implemented"))
    }
    fn grant_inspect(&self, _: &str, _: &str) -> CompatResult<Value> {
        Err(CompatError::new(501, "not_implemented"))
    }
    fn grant_issue(&self, _: &str, _: &str) -> CompatResult<Value> {
        Err(CompatError::new(501, "not_implemented"))
    }
    fn grant_revoke_user(&self, _: &str, _: &str, _: &str) -> CompatResult<Value> {
        Err(CompatError::new(501, "not_implemented"))
    }
    fn grant_revoke(&self, _: &str, _: &str) -> CompatResult<Value> {
        Err(CompatError::new(501, "not_implemented"))
    }
}
