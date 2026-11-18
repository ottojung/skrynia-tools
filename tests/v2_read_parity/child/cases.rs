use serde_json::{Value, json};
use skrynia_tools::wire::v2compat::grants::Authority;
fn request(loc: &str, ops: Value) -> Value {
    json!({"locations":loc,"operations":ops,"maxBytes":4000,"maxObjects":2})
}
pub fn verify(url: &str, token: &str, authority: &Authority) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    let mut invalid = vec![
        json!([]),
        json!({"operations":[]}),
        request("outsider/*", json!(["create"])),
        request("team/*", json!(["modify"])),
        request("team/*", json!(["unknown"])),
        request("team/*", json!(["create", "create"])),
        request("team/*", json!([])),
        request("team/**", json!(["create"])),
    ];
    invalid[6]["maxBytes"] = json!(2047);
    for value in invalid {
        let expected = authority
            .child_spec(token, &value)
            .err()
            .ok_or("fixture unexpectedly valid")?;
        let res = client
            .post(format!("{url}/grants"))
            .bearer_auth(token)
            .json(&value)
            .send()
            .map_err(|e| e.to_string())?;
        let status = res.status().as_u16();
        let body: Value = res.json().map_err(|e| e.to_string())?;
        if status != expected.status || body != json!({"error": expected.code}) {
            return Err(format!(
                "v2 child spec {value}: expected {} {} got {status} {body}",
                expected.status, expected.code
            ));
        }
    }
    Ok(())
}
