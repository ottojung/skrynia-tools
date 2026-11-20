mod config;
mod headers;
mod redirect;
mod token;
#[cfg(unix)]
fn private(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
