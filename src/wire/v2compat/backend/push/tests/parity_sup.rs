use super::{CompatError, GrantAuth, Ids, Request, scope};
use std::collections::BTreeMap;
pub struct G;
impl GrantAuth for G {
    fn can_subscribe(&self, token: &str, loc: &str) -> Result<String, CompatError> {
        if token != &"b".repeat(64) {
            return Err(CompatError::new(403, "invalid_grant_user_token"));
        }
        if !scope::subset(loc, "apps/*") {
            return Err(CompatError::new(403, "location_not_authorized"));
        }
        Ok("g1".into())
    }
}
pub struct I(pub u32);
impl Ids for I {
    fn id(&self) -> String {
        "s1".into()
    }
    fn secret(&self) -> String {
        format!("{:064x}", self.0)
    }
}
pub fn sreq(tok: &str, body: serde_json::Value) -> Request {
    let mut h = BTreeMap::new();
    h.insert("authorization".into(), format!("Bearer {tok}"));
    Request {
        method: "POST".into(),
        path: "/push/subscriptions".into(),
        query: None,
        headers: h,
        body: serde_json::to_vec(&body).unwrap(),
    }
}
