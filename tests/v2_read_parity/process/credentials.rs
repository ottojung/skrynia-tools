use super::Server;

impl Server {
    pub fn root_grant(&self) -> Result<(String, String), String> {
        super::root_file::credentials(&self.dir.join("state/grants"))
    }
    pub fn root_user(&self) -> Result<String, String> {
        let (id, issuer) = super::root_file::credentials(&self.dir.join("state/grants"))?;
        let url = format!("{}/grants/{id}/users", self.url);
        let reply = reqwest::blocking::Client::new()
            .post(url)
            .bearer_auth(&issuer)
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
