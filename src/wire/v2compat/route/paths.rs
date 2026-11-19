pub fn known(path: &str) -> bool {
    matches!(path, "/health" | "/version" | "/push/vapid" | "/store")
        || path.starts_with("/store/")
        || path.starts_with("/push")
}
