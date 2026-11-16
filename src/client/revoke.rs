use super::types::SkryniaClient;
use anyhow::{Result, bail};
use reqwest::StatusCode;
impl SkryniaClient {
    pub fn revoke_user(&self, id: &str, user_id: &str, issuer: &str) -> Result<()> {
        let request = self
            .http
            .delete(self.path(&["grants", id, "users", user_id])?)
            .bearer_auth(issuer);
        let response = Self::mutating(request.send())?;
        Self::accept_revocation(response, &["user_not_found", "grant_not_found"])
    }
    pub fn revoke_grant(&self, id: &str, issuer: &str) -> Result<()> {
        let request = self
            .http
            .delete(self.path(&["grants", id])?)
            .bearer_auth(issuer);
        let response = Self::mutating(request.send())?;
        Self::accept_revocation(response, &["grant_not_found"])
    }
}

impl SkryniaClient {
    fn accept_revocation(response: reqwest::blocking::Response, missing: &[&str]) -> Result<()> {
        if response.status() == StatusCode::NOT_FOUND {
            let body = Self::json_response(response)?;
            if missing
                .iter()
                .any(|code| body["error"].as_str() == Some(*code))
            {
                return Ok(());
            }
            bail!("unexpected 404 during revocation; authority state cannot be confirmed");
        }
        Self::committed(response, StatusCode::OK, "revoke")?;
        Ok(())
    }
}
