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
}
