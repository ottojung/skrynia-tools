use super::{CompatError, GrantAuth, Ids, Request, scope};
use std::collections::BTreeMap;
pub struct G(pub String, pub String);
impl GrantAuth for G {
    fn can_subscribe(&self, token: &str, loc: &str) -> Result<String, CompatError> {
        if token != self.0 {
            return Err(CompatError::new(403, "invalid_grant_user_token"));
        }
        if !scope::subset(loc, &self.1) {
            return Err(CompatError::new(403, "location_not_authorized"));
        }
        Ok("g1".into())
    }
}
pub struct I(pub u32);
impl Ids for I {
    fn id(&self) -> String {
        format!("id{:030}", self.0)
    }
    fn secret(&self) -> String {
        format!("{:064x}", self.0)
    }
}
pub fn req(tok: Option<&str>, cap: Option<&str>, body: serde_json::Value) -> Request {
    let mut h = BTreeMap::new();
    if let Some(t) = tok {
        h.insert("authorization".into(), format!("Bearer {t}"));
    }
    if let Some(c) = cap {
        h.insert("x-skrynia-capability".into(), c.into());
    }
    Request {
        method: "POST".into(),
        path: "/push/subscriptions".into(),
        query: None,
        headers: h,
        body: serde_json::to_vec(&body).unwrap(),
    }
}
