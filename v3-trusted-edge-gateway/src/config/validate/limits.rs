use super::Config;

pub fn validate(cfg: &Config) -> Result<(), String> {
    if cfg.max_concurrent_requests == 0 {
        return Err("max_concurrent_requests must be > 0".into());
    }
    for (name, val) in [
        ("acquire_timeout_ms", cfg.acquire_timeout_ms),
        ("request_timeout_ms", cfg.request_timeout_ms),
        ("upload_idle_timeout_ms", cfg.upload_idle_timeout_ms),
        (
            "upstream_connect_timeout_ms",
            cfg.upstream_connect_timeout_ms,
        ),
        ("upstream_read_timeout_ms", cfg.upstream_read_timeout_ms),
    ] {
        if val == 0 {
            return Err(format!("{name} must be > 0"));
        }
    }
    Ok(())
}
