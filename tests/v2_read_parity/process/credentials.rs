use super::Server;
use std::fs;

impl Server {
    pub fn root_user(&self) -> Result<String, String> {
        let dir = self.dir.join("state/grants");
        let entry = fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .next()
            .ok_or("missing root grant")?
            .map_err(|e| e.to_string())?;
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let id = value["id"].as_str().ok_or("missing root id")?;
        let issuer = value["rootIssuerToken"]
            .as_str()
            .ok_or("missing root issuer")?;
        let url = format!("{}/grants/{id}/users", self.url);
        let reply = reqwest::blocking::Client::new()
            .post(url)
            .bearer_auth(issuer)
            .send()
            .map_err(|e| e.to_string())?;
        if reply.status() != 200 {
            return Err(format!("user issue: {}", reply.status()));
        }
        let json: serde_json::Value = reply.json().map_err(|e| e.to_string())?;
        json["userToken"]
            .as_str()
            .map(str::to_owned)
            .ok_or("missing userToken".into())
    }
}
