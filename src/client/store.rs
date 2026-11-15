use super::types::{CreateResult, SkryniaClient, Store};
use anyhow::Result;
use reqwest::StatusCode;

impl Store for SkryniaClient {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.read(key)
    }
    fn create(&self, key: &str, data: &[u8]) -> Result<CreateResult> {
        let req = self
            .http
            .post(self.path(&["store", key])?)
            .body(data.to_vec());
        let response = Self::mutating(self.auth(req)?.send())?;
        if response.status() == StatusCode::CONFLICT {
            return Ok(CreateResult::Exists);
        }
        Self::success(response)?;
        Ok(CreateResult::Created)
    }
}
impl SkryniaClient {
    pub fn replace(&self, key: &str, data: &[u8], digest: Option<&str>) -> Result<()> {
        let mut req = self
            .auth(self.http.put(self.path(&["store", key])?))?
            .body(data.to_vec());
        if let Some(digest) = digest {
            req = req.header("X-Skrynia-If-SHA256", digest);
        }
        Self::success(Self::mutating(req.send())?)?;
        Ok(())
    }
    pub fn delete(&self, key: &str, digest: Option<&str>) -> Result<()> {
        let mut req = self.auth(self.http.delete(self.path(&["store", key])?))?;
        if let Some(digest) = digest {
            req = req.header("X-Skrynia-If-SHA256", digest);
        }
        Self::success(Self::mutating(req.send())?)?;
        Ok(())
    }
}
