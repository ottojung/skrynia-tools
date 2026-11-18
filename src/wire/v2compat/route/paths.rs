pub fn stub(method: &str, path: &str) -> bool {
    matches!(
        (method, path),
        ("POST", "/grants") | ("POST", "/push/subscriptions")
    ) || (method == "GET" && path.starts_with("/grants/"))
        || (method == "POST" && path.starts_with("/grants/") && path.ends_with("/users"))
        || (method == "DELETE" && path.starts_with("/grants/"))
        || ((method == "PUT" || method == "DELETE") && path.starts_with("/push/subscriptions/"))
}
pub fn known(path: &str) -> bool {
    matches!(path, "/health" | "/version" | "/push/vapid" | "/store")
        || path.starts_with("/store/")
        || path.starts_with("/grants")
        || path.starts_with("/push")
}
