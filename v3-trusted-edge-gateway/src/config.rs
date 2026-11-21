use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: String,
    pub upstream: String,
    pub operator_token_file: String,
    /// Maximum request/response body bytes forwarded (default 16 MiB).
    #[serde(default = "default_max_body")]
    pub max_body_bytes: usize,
    pub apps: Vec<AppConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub name: String,
    pub hosts: Vec<String>,
    pub origins: Vec<String>,
    /// Edge client/session/capability secrets (separate from root operator token).
    /// At least one is required.
    pub edge_capabilities: Vec<String>,
}

fn default_max_body() -> usize {
    16 * 1024 * 1024
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
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
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<(), String> {
        if self.bind.trim().is_empty() || self.upstream.trim().is_empty() {
            return Err("bind and upstream are required".into());
        }
        let bind: SocketAddr = self.bind.parse().map_err(|_| "invalid bind address")?;
        if !bind.ip().is_loopback() {
            return Err("bind must be loopback-only".into());
        }
        let upstream = reqwest::Url::parse(&self.upstream).map_err(|_| "invalid upstream url")?;
        let host = upstream.host_str().ok_or("invalid upstream host")?;
        let loopback = if host.eq_ignore_ascii_case("localhost") {
            true
        } else {
            host.parse::<std::net::IpAddr>()
                .map(|ip| ip.is_loopback())
                .unwrap_or(false)
        };
        if !loopback {
            return Err("upstream must be loopback-only".into());
        }
        if self.max_body_bytes == 0 {
            return Err("max_body_bytes must be > 0".into());
        }
        if self.apps.is_empty() {
            return Err("at least one app must be configured".into());
        }
        let mut seen_origin: BTreeMap<String, String> = BTreeMap::new();
        let mut seen_host: BTreeMap<String, String> = BTreeMap::new();
        for app in &self.apps {
            if app.name.trim().is_empty() {
                return Err("app name is required".into());
            }
            if app.hosts.is_empty() || app.origins.is_empty() || app.edge_capabilities.is_empty() {
                return Err(format!(
                    "app {} requires hosts, origins, and edge capabilities",
                    app.name
                ));
            }
            for o in &app.origins {
                let origin = normalize_origin(o)
                    .ok_or_else(|| format!("invalid origin for app {}", app.name))?;
                if let Some(prev) = seen_origin.insert(origin.clone(), app.name.clone()) {
                    if prev != app.name {
                        return Err(format!("origin {origin} mapped to multiple apps"));
                    }
                }
            }
            for h in &app.hosts {
                let host = normalize_host(h)
                    .ok_or_else(|| format!("invalid host for app {}", app.name))?;
                if let Some(prev) = seen_host.insert(host.clone(), app.name.clone()) {
                    if prev != app.name {
                        return Err(format!("host {host} mapped to multiple apps"));
                    }
                }
            }
            for c in &app.edge_capabilities {
                if c.is_empty() {
                    return Err(format!("empty edge capability for app {}", app.name));
                }
            }
        }
        // basic sanity: operator token file path must exist as a path string
        if !Path::new(&self.operator_token_file).exists() {
            return Err("operator token file missing".into());
        }
        Ok(())
    }

    pub fn find_app(&self, host: &str, origin: &str) -> Option<&AppConfig> {
        let host = normalize_host(host)?;
        let origin = normalize_origin(origin)?;
        self.apps.iter().find(|app| {
            app.hosts
                .iter()
                .any(|h| normalize_host(h).as_deref() == Some(host.as_str()))
                && app
                    .origins
                    .iter()
                    .any(|o| normalize_origin(o).as_deref() == Some(origin.as_str()))
        })
    }
}

pub fn normalize_host(host: &str) -> Option<String> {
    let h = host.trim();
    if h.is_empty() || h.contains('/') || h.contains(' ') || h.contains('\\') {
        return None;
    }
    let lower = h.to_ascii_lowercase();
    // allow host or host:port; reject userinfo
    if lower.contains('@') {
        return None;
    }
    Some(lower)
}

pub fn normalize_origin(origin: &str) -> Option<String> {
    let o = origin.trim();
    let (scheme, rest) = o.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "https" && scheme != "http" {
        return None;
    }
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('@')
    {
        return None;
    }
    Some(format!("{scheme}://{}", rest.to_ascii_lowercase()))
}
