//! Environment configuration. The listener is loopback-only by construction;
//! there is no CLI, argument parser or public network exposure.

use super::roots::{self, Roots};
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Config {
    pub bind: SocketAddr,
    pub roots: Roots,
}

pub fn load() -> Result<Config, String> {
    let raw =
        std::env::var("PUBLICATION_PROVIDER_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let bind: SocketAddr = raw
        .parse()
        .map_err(|e| format!("invalid bind address: {e}"))?;
    if !bind.ip().is_loopback() {
        return Err("bind must be a loopback address".into());
    }
    let requests = required("PATH_TO_PUBLICATION_REQUESTS")?;
    let disk = required("PATH_TO_SKRYNIA_DISK")?;
    let nginx = required("PATH_TO_NGINX_SKRYNIA_ROOT")?;
    Ok(Config {
        bind,
        roots: roots::open(&requests, &disk, &nginx)?,
    })
}

fn required(name: &str) -> Result<PathBuf, String> {
    let value = std::env::var(name).map_err(|_| format!("{name} is required"))?;
    if value.is_empty() {
        return Err(format!("{name} is empty"));
    }
    Ok(PathBuf::from(value))
}
