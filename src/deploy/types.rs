use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct DeploySpec {
    pub name: String,
    pub repo: String,
    pub commit: Option<String>,
    pub subdir: String,
    pub apps_dir: PathBuf,
    pub builder_image: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub commit: String,
    pub subdir: String,
    pub builder_image: String,
}
impl DeploySpec {
    pub fn validate(&self) -> Result<()> {
        if !super::validate::valid_name(&self.name) || !super::validate::valid_rel(&self.subdir) {
            bail!("invalid app name or source subdirectory");
        }
        super::validate::validate_repo(&self.repo)?;
        if self
            .commit
            .as_ref()
            .is_some_and(|c| c.len() != 40 || !c.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            bail!("commit must be a full 40-character Git SHA-1");
        }
        if self.builder_image.trim().is_empty() {
            bail!("builder image is required");
        }
        Ok(())
    }
}
