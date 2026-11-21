use super::model::Config;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn load(path: &str) -> Result<Config, String> {
    let meta = fs::metadata(path).map_err(|_| "failed to read config file")?;
    if !meta.file_type().is_file() {
        return Err("config must be a regular file".into());
    }
    #[cfg(unix)]
    if meta.permissions().mode() & 0o077 != 0 {
        return Err("config file permissions too open".into());
    }
    let raw = fs::read_to_string(path).map_err(|_| "failed to read config file")?;
    let cfg: Config = serde_json::from_str(&raw).map_err(|_| "invalid config")?;
    super::validate::validate(&cfg)?;
    Ok(cfg)
}
