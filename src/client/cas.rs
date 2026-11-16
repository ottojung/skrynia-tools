use super::types::SkryniaClient;
use anyhow::Result;
use reqwest::StatusCode;

impl SkryniaClient {
    pub fn compare_replace(&self, key: &str, data: &[u8], expected_sha: &str) -> Result<bool> {
        let req = self
            .auth(self.http.put(self.object_path(key)?))?
            .header("X-Skrynia-If-SHA256", expected_sha)
            .body(data.to_vec());
        let response = Self::mutating(req.send())?;
        if response.status() == StatusCode::PRECONDITION_FAILED {
            return Ok(false);
        }
        Self::committed(response, StatusCode::OK, "compare_replace")?;
        Ok(true)
    }
}
