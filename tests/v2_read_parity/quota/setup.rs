use super::Server;
use serde_json::{Value, json};
pub fn token(server: &Server) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let root = server.root_user()?;
    let created = client
        .post(format!("{}/grants", server.url))
        .bearer_auth(&root)
        .json(&json!({"locations":"team/*",
            "operations":["create","modify","delete"],
            "maxBytes":3000,"maxObjects":1}))
        .send()
        .map_err(|e| e.to_string())?;
    if created.status() != 200 {
        return Err(format!("child status {}", created.status()));
    }
    let value: Value = created.json().map_err(|e| e.to_string())?;
    let id = value["id"].as_str().ok_or("missing child id")?;
    let issuer = value["issuerToken"]
        .as_str()
        .ok_or("missing child issuer")?;
    let res = client
        .post(format!("{}/grants/{id}/users", server.url))
        .bearer_auth(issuer)
        .send()
        .map_err(|e| e.to_string())?;
    if res.status() != 200 {
        return Err(format!("user status {}", res.status()));
    }
    let user: Value = res.json().map_err(|e| e.to_string())?;
    user["userToken"]
        .as_str()
        .map(str::to_owned)
        .ok_or("child token not returned".into())
}
