//! Safe local config loading. Secrets are never logged or formatted.
mod check;
mod field;
mod hex;
mod parse;
mod pump;
mod read;
use std::path::{Path, PathBuf};

pub(super) use super::{Config, Edge, Error, Result, self_uid};

pub struct RuntimeConfig {
    pub edge: Edge,
    pub pump: Config,
    pub vapid_private_key: [u8; 32],
    pub ledger_dir: PathBuf,
    pub poll_ms: u64,
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self> {
        let path =
            std::env::var("WEBPUSH_SENDER_CONFIG").map_err(|_| Error::Config("config-env"))?;
        Self::from_path(Path::new(&path))
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let bytes = read::read_file(path)?;
        let v: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| Error::Config("config-json"))?;
        parse::parse(&v)
    }
}
