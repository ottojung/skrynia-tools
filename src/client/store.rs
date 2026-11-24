use super::{CreateResult, SkryniaClient, Store, native};
use anyhow::Result;

impl Store for SkryniaClient {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.read(key)
    }
    fn create(&self, key: &str, data: &[u8]) -> Result<CreateResult> {
        let frame = native::put(self.caps.writer()?, key, None, data)?;
        Ok(if native::written(&self.post(frame)?, data)? {
            CreateResult::Created
        } else {
            CreateResult::Exists
        })
    }
}
impl SkryniaClient {
    pub fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        native::read(&self.post(native::get(&self.caps.read, key)?)?)
    }
    pub fn digest(data: &[u8]) -> String {
        hex::encode(native::digest(data))
    }
    pub fn compare_replace(&self, key: &str, data: &[u8], expected_sha: &str) -> Result<bool> {
        let expected = native::parse_cap(expected_sha)?;
        let frame = native::put(self.caps.writer()?, key, Some(&expected), data)?;
        native::written(&self.post(frame)?, data)
    }
    pub fn delete(&self, key: &str, digest: &str) -> Result<bool> {
        let frame = native::delete(self.caps.writer()?, key, &native::parse_cap(digest)?)?;
        native::deleted(&self.post(frame)?)
    }
}
