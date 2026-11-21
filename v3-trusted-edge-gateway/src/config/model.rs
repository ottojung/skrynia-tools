use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: String,
    pub upstream: String,
    pub operator_token_file: String,
    #[serde(default = "super::defaults::max_body")]
    pub max_body_bytes: usize,
    #[serde(default = "super::defaults::concurrent")]
    pub max_concurrent_requests: usize,
    #[serde(default = "super::defaults::acquire_ms")]
    pub acquire_timeout_ms: u64,
    #[serde(default = "super::defaults::request_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "super::defaults::upload_idle_ms")]
    pub upload_idle_timeout_ms: u64,
    #[serde(default = "super::defaults::upstream_connect_ms")]
    pub upstream_connect_timeout_ms: u64,
    #[serde(default = "super::defaults::upstream_read_ms")]
    pub upstream_read_timeout_ms: u64,
    pub apps: Vec<AppConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub name: String,
    pub hosts: Vec<String>,
    pub origins: Vec<String>,
    pub edge_capabilities: Vec<String>,
}
