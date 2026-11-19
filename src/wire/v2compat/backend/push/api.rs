use super::{
    CompatError,
    ops::{self, GrantAuth, Ids},
    reply,
    store::Store,
};
pub fn subscribe<S: Store, G: GrantAuth, I: Ids>(
    s: &mut S,
    g: &G,
    ids: &I,
    req: &super::Request,
) -> super::Response {
    match ops::subscribe(s, g, ids, req) {
        Ok(v) => reply::json(
            if v.deduped { 200 } else { 201 },
            serde_json::to_vec(&v).unwrap_or_default(),
        ),
        Err(e) => err(e),
    }
}
pub fn update<S: Store>(s: &mut S, id: &str, req: &super::Request) -> super::Response {
    match ops::update(s, id, req) {
        Ok(()) => empty(200),
        Err(e) => err(e),
    }
}
pub fn delete<S: Store>(s: &mut S, id: &str, req: &super::Request) -> super::Response {
    match ops::delete(s, id, req) {
        Ok(()) => empty(200),
        Err(e) => err(e),
    }
}
fn empty(status: u16) -> super::Response {
    reply::json(status, b"{}".to_vec())
}
fn err(e: CompatError) -> super::Response {
    reply::error(e.status, &e.code)
}
