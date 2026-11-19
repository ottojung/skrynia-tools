use super::*;
use serde_json::json;
impl<S: StateStore, R: RandomSource> GrantAdmin for DurableBackend<S, R> {
    fn grant_create(&self, token: &str, spec: &Value) -> CompatResult<Value> {
        let mut value = self.create_grant(token, spec)?;
        let fields = ["locations", "operations", "maxBytes", "maxObjects"];
        for name in fields {
            value[name] = spec[name].clone();
        }
        Ok(value)
    }
    fn grant_inspect(&self, id: &str, token: &str) -> CompatResult<Value> {
        super::inspect::run(self, id, token)
    }
    fn grant_issue(&self, id: &str, token: &str) -> CompatResult<Value> {
        self.issue_user(id, token)
    }
    fn grant_revoke_user(&self, id: &str, user: &str, token: &str) -> CompatResult<Value> {
        self.revoke_user(id, user, token)?;
        Ok(json!({"ok":true,"grantId":id,"userId":user}))
    }
    fn grant_revoke(&self, id: &str, token: &str) -> CompatResult<Value> {
        self.revoke_grant(id, token)?;
        Ok(json!({"ok":true,"grantId":id}))
    }
}
