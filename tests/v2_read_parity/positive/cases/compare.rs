use super::*;
pub(crate) fn compare(url: &str, store: &RootStore, req: Request) -> Result<(), String> {
    let expected = handle(store, &req);
    let client = reqwest::blocking::Client::new();
    let mut http = client
        .request(req.method.parse().unwrap(), format!("{url}{}", req.path))
        .body(req.body.clone());
    for (name, value) in &req.headers {
        http = http.header(name, value);
    }
    let got = http.send().map_err(|e| e.to_string())?;
    let status = got.status().as_u16();
    let headers = got.headers().clone();
    let bytes = got.bytes().map_err(|e| e.to_string())?;
    if status != expected.status || bytes != expected.body {
        return Err(format!(
            "{} {} expected {} {:?} got {status} {:?}",
            req.method, req.path, expected.status, expected.body, bytes
        ));
    }
    for key in ["cache-control", "content-type", "content-length"] {
        let actual = headers.get(key).and_then(|v| v.to_str().ok());
        let wanted = expected.headers.get(key).map(String::as_str);
        if actual != wanted {
            return Err(format!("{} {key} {actual:?} != {wanted:?}", req.method));
        }
    }
    Ok(())
}
