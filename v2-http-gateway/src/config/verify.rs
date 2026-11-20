use std::net::SocketAddr;
pub(crate) fn check(bind: SocketAddr, endpoint: &reqwest::Url) -> Result<(), String> {
    if !bind.ip().is_loopback() {
        return Err("bind must be loopback".into());
    }
    let local = matches!(endpoint.host_str(), Some("127.0.0.1" | "::1" | "[::1]"));
    if endpoint.scheme() != "http"
        || !local
        || endpoint.path() != "/"
        || endpoint.query().is_some()
        || endpoint.port().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.fragment().is_some()
    {
        return Err("upstream must be http loopback with explicit port at /".into());
    }
    Ok(())
}
