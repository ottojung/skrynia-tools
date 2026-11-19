use super::Backend;
use serde_json::Value;
use skrynia_tools::wire::v2compat::{Request, handle};
pub fn call(
    b: &Backend,
    method: &str,
    path: &str,
    token: &str,
    body: &[u8],
) -> Result<(u16, Value), String> {
    let mut req = Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: Default::default(),
        body: body.to_vec(),
    };
    if !token.is_empty() {
        req.headers
            .insert("authorization".into(), format!("Bearer {token}"));
    }
    let response = handle(b, &req);
    let body = serde_json::from_slice(&response.body).map_err(|e| e.to_string())?;
    Ok((response.status, body))
}
