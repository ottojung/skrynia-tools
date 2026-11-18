use super::Server;
use serde_json::{Value, json};
pub fn child_user(server: &Server, root_token: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let grant = client
        .post(format!("{}/grants", server.url))
        .bearer_auth(root_token)
        .json(&json!({"locations":"team/*",
            "operations":["create","delegate"],
            "maxBytes":10000,"maxObjects":10}))
        .send()
        .map_err(|e| e.to_string())?;
    if grant.status() != 200 {
        return Err(format!("child grant creation HTTP {}", grant.status()));
    }
    let grant: Value = grant.json().map_err(|e| e.to_string())?;
    let id = grant["id"].as_str().ok_or("child grant id missing")?;
    let issuer = grant["issuerToken"]
        .as_str()
        .ok_or("child issuer missing")?;
    let res = client
        .post(format!("{}/grants/{id}/users", server.url))
        .bearer_auth(issuer)
        .send()
        .map_err(|e| e.to_string())?;
    if res.status() != 200 {
        return Err(format!("child token issuance HTTP {}", res.status()));
    }
    let value: Value = res.json().map_err(|e| e.to_string())?;
    value["userToken"]
        .as_str()
        .map(str::to_owned)
        .ok_or("child user token missing".into())
}
