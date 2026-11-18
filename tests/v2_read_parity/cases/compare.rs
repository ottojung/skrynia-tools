use super::*;
pub fn one(
    client: &reqwest::blocking::Client,
    obj: &MemoryStore,
    url: &str,
    method: &str,
    path: &str,
    header: &str,
    value: &str,
) -> Result<(), String> {
    let mut headers = BTreeMap::new();
    if !header.is_empty() {
        headers.insert(header.to_string(), value.to_string());
    }
    let req = Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers,
        body: vec![],
    };
    let expected = handle(obj, &req);
    let mut http = client.request(method.parse().unwrap(), format!("{url}{path}"));
    if !header.is_empty() {
        http = http.header(header, value);
    }
    let received = http.send().map_err(|e| e.to_string())?;
    let status = received.status().as_u16();
    let real_headers = received.headers().clone();
    let body = received.bytes().map_err(|e| e.to_string())?;
    super::match_http::check(
        status,
        &body,
        &real_headers,
        &expected,
        &format!("{method} {path} {header}={value}"),
    )?;
    Ok(())
}
