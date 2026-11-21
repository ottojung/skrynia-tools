use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: String,
    pub upstream: String,
    pub operator_token_file: String,
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
    pub edge_capabilities: Vec<String>,
}

fn default_max_body() -> usize {
    16 * 1024 * 1024
}
