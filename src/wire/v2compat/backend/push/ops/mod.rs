mod sub;
pub(super) use super::{
    CompatError, Record, SubView, auth as pauth, parse as pparse, store, validate as pvalidate,
};
use super::{Request, auth, parse, store::Store, validate};
pub use sub::subscribe;
pub trait Ids {
    fn id(&self) -> String;
    fn secret(&self) -> String;
}
pub trait GrantAuth {
    fn can_subscribe(&self, token: &str, locations: &str) -> Result<String, CompatError>;
}
pub fn update<S: Store>(s: &mut S, id: &str, req: &Request) -> Result<(), CompatError> {
    let token = auth::cap_header(req)?;
    let rec = s
        .get(id)
        .ok_or_else(|| CompatError::new(404, "subscription_not_found"))?;
    auth::capability(&rec, token)?;
    let body = parse::update(&req.body)?;
    validate::validate_update(&body)?;
    s.update_target(id, body.endpoint, body.keys)
}
pub fn delete<S: Store>(s: &mut S, id: &str, req: &Request) -> Result<(), CompatError> {
    let token = auth::cap_header(req)?;
    let rec = s
        .get(id)
        .ok_or_else(|| CompatError::new(404, "subscription_not_found"))?;
    auth::capability(&rec, token)?;
    s.remove(id)
}
