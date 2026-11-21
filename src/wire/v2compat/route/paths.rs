pub fn stub(method: &str, path: &str) -> bool {
    matches!((method, path), ("POST", "/push/subscriptions"))
        || ((method == "PUT" || method == "DELETE") && path.starts_with("/push/subscriptions/"))
}
pub fn known(path: &str) -> bool {
    matches!(path, "/health" | "/version" | "/push/vapid" | "/store")
        || path.starts_with("/store/")
        || path.starts_with("/grants")
        || path.starts_with("/push")
}
