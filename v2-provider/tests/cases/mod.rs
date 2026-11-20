mod errors;
mod fake;
mod read;
mod wire;
pub use fake::Fake;
use skrynia_tools::wire::v2compat::Request;
use std::collections::BTreeMap;

pub fn req(method: &str, path: &str) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: BTreeMap::new(),
        body: Vec::new(),
    }
}
