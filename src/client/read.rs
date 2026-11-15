use super::{errors::ClientError, types::SkryniaClient};
use anyhow::Result;
use sha2::{Digest, Sha256};

impl SkryniaClient {
    pub fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let response = self
            .http
            .get(self.object_path(key)?)
            .send()
            .map_err(|source| ClientError::Transport { source })?;
        if Self::not_found(response.status()) {
            return Ok(None);
        }
        Ok(Some(Self::success(response)?.bytes()?.to_vec()))
    }
    pub fn digest(data: &[u8]) -> String {
        hex::encode(Sha256::digest(data))
    }
    pub fn list_page(&self, matcher: &str, after: Option<&str>) -> Result<serde_json::Value> {
        if !super::wire::valid_matcher(matcher) {
            anyhow::bail!("invalid Skrynia locations matcher");
        }
        let mut url = self.path(&["store"])?;
        url.query_pairs_mut()
            .append_pair("locations", matcher)
            .append_pair("limit", "1000");
        if let Some(after) = after {
            url.query_pairs_mut().append_pair("after", after);
        }
        let req = self.auth(self.http.get(url))?;
        Ok(Self::success(
            req.send()
                .map_err(|source| ClientError::Transport { source })?,
        )?
        .json()?)
    }
}
