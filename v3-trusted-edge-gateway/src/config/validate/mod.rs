mod apps;
mod checks;
mod net;

pub(super) use super::model::AppConfig;
pub(super) use super::normalize::{normalize_host, normalize_origin};

use super::model::Config;
use std::path::Path;

pub fn validate(cfg: &Config) -> Result<(), String> {
    if cfg.bind.trim().is_empty() || cfg.upstream.trim().is_empty() {
        return Err("bind and upstream are required".into());
    }
    net::validate_bind(&cfg.bind)?;
    net::validate_upstream(&cfg.upstream)?;
    if cfg.max_body_bytes == 0 {
        return Err("max_body_bytes must be > 0".into());
    }
    if cfg.apps.is_empty() {
        return Err("at least one app must be configured".into());
    }
    apps::validate_apps(&cfg.apps)?;
    if !Path::new(&cfg.operator_token_file).exists() {
        return Err("operator token file missing".into());
    }
    Ok(())
}
