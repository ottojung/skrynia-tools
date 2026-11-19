use serde_json::Value;
pub fn call(
    url: &str,
    method: &str,
    path: &str,
    token: &str,
    body: &[u8],
) -> Result<(u16, Value), String> {
    let client = reqwest::blocking::Client::new();
    let mut req = client.request(method.parse().unwrap(), format!("{url}{path}"));
    if !token.is_empty() {
        req = req.bearer_auth(token);
    }
    let resp = req.body(body.to_vec()).send().map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body: Value = resp.json().map_err(|e| e.to_string())?;
    Ok((status, body))
}
