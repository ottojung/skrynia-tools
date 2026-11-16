use super::types::SkryniaClient;
use anyhow::Result;
use reqwest::StatusCode;

impl SkryniaClient {
    pub fn create_grant(
        &self,
        matcher: &str,
        operations: &[&str],
        max_bytes: u64,
        max_objects: u64,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({"locations": matcher, "operations": operations,
            "maxBytes": max_bytes, "maxObjects": max_objects});
        let req = self
            .auth(self.http.post(self.path(&["grants"])?))?
            .json(&body);
        Self::json_response(Self::committed(
            Self::mutating(req.send())?,
            StatusCode::OK,
            "create_grant",
        )?)
    }
    pub fn issue_user(&self, grant: &str, issuer_token: &str) -> Result<serde_json::Value> {
        let req = self
            .http
            .post(self.path(&["grants", grant, "users"])?)
            .bearer_auth(issuer_token);
        Self::json_response(Self::committed(
            Self::mutating(req.send())?,
            StatusCode::OK,
            "issue_user",
        )?)
    }
}
