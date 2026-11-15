use super::{types::SkryniaClient, wire};
use anyhow::{Result, bail};
use reqwest::Url;

impl SkryniaClient {
    pub fn valid_key(key: &str) -> bool {
        wire::valid_key(key)
    }
    pub(super) fn object_path(&self, key: &str) -> Result<Url> {
        if !wire::valid_key(key) {
            bail!("invalid Skrynia object key");
        }
        self.path(&["store", key])
    }
    pub(super) fn path(&self, segments: &[&str]) -> Result<Url> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid base URL"))?
            .extend(segments);
        Ok(url)
    }
}
