mod http;
mod local;
use super::fixture::Backend;
use serde_json::Value;
type Pair = ((u16, Value), (u16, Value));
pub struct Endpoint<'a> {
    pub path: &'a str,
    pub token: &'a str,
}
pub fn each(
    url: &str,
    b: &Backend,
    method: &str,
    path: &str,
    t2: &str,
    t3: &str,
    data: &[u8],
) -> Result<Pair, String> {
    at(
        url,
        b,
        method,
        Endpoint { path, token: t2 },
        Endpoint { path, token: t3 },
        data,
    )
}
pub fn at(
    url: &str,
    b: &Backend,
    method: &str,
    v2: Endpoint<'_>,
    v3: Endpoint<'_>,
    data: &[u8],
) -> Result<Pair, String> {
    Ok((
        http::call(url, method, v2.path, v2.token, data)?,
        local::call(b, method, v3.path, v3.token, data)?,
    ))
}
pub fn equal((left, right): Pair) -> Result<(), String> {
    if left != right {
        return Err(format!("v2 {left:?} != facade {right:?}"));
    }
    Ok(())
}
