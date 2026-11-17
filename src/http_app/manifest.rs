use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub path: String,
    pub key: String,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub schema: u8,
    pub name: String,
    pub id: String,
    pub commit: String,
    pub repo: String,
    pub subdir: String,
    pub files: Vec<Asset>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pointer {
    pub schema: u8,
    pub release: String,
    pub nonce: String,
}
impl Pointer {
    pub fn new(release: &str) -> Self {
        Self {
            schema: 1,
            release: release.into(),
            nonce: format!("{:032x}", rand::random::<u128>()),
        }
    }
}
