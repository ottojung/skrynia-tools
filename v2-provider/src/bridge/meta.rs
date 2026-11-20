use super::{Callback, InvokeBackend};
use skrynia_tools::wire::v2compat::{Meta, Response};
use std::collections::BTreeMap;
fn unavailable() -> Response {
    let body = b"{\"error\":\"unavailable\"}".to_vec();
    let mut headers = BTreeMap::new();
    headers.insert("content-type".into(), "application/json".into());
    headers.insert("content-length".into(), body.len().to_string());
    Response {
        status: 503,
        headers,
        body,
    }
}
impl<I: Callback> Meta for InvokeBackend<'_, I> {
    fn health(&self) -> Response {
        unavailable()
    }
    fn version(&self) -> Response {
        unavailable()
    }
    fn push_vapid(&self) -> Response {
        unavailable()
    }
}
