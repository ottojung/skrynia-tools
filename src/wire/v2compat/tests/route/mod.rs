use super::{Request, Unavailable, handle};
use std::collections::BTreeMap;
mod methods;
mod mutations;
mod preflight;
fn req(method: &str, path: &str) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: BTreeMap::new(),
        body: Vec::new(),
    }
}
