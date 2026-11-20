use super::{Response, route_reply};
pub fn json(body: Vec<u8>) -> Response {
    route_reply::json(200, body)
}
pub fn empty() -> Response {
    route_reply::json(200, b"{}".to_vec())
}
pub fn map<T, F>(r: Result<T, crate::wire::v2compat::backend::CompatError>, f: F) -> Response
where
    F: FnOnce(T) -> Response,
{
    match r {
        Ok(v) => f(v),
        Err(e) => route_reply::error(e.status, &e.code),
    }
}
