use std::net::SocketAddr;
use std::path::Path;

pub fn build(
    addr: SocketAddr,
    kernel_url: &str,
    token_path: &Path,
    apps: serde_json::Value,
    overrides: serde_json::Value,
) -> serde_json::Value {
    let mut cfg = serde_json::json!({
        "bind": addr.to_string(),
        "upstream": kernel_url,
        "operator_token_file": token_path.to_string_lossy(),
        "max_body_bytes": 16 * 1024 * 1024,
        "apps": apps
    });
    merge(&mut cfg, overrides);
    cfg
}

pub fn set_mode(path: impl AsRef<Path>, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
}

fn merge(base: &mut serde_json::Value, over: serde_json::Value) {
    if let (serde_json::Value::Object(b), serde_json::Value::Object(o)) = (base, over) {
        for (k, v) in o {
            b.insert(k, v);
        }
    }
}
