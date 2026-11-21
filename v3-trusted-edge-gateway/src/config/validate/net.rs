use std::net::{IpAddr, SocketAddr};

pub fn validate_bind(bind: &str) -> Result<(), String> {
    let addr: SocketAddr = bind.parse().map_err(|_| "invalid bind address")?;
    if !addr.ip().is_loopback() {
        return Err("bind must be loopback-only".into());
    }
    Ok(())
}

pub fn validate_upstream(upstream: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(upstream).map_err(|_| "invalid upstream url")?;
    if url.scheme() != "http" {
        return Err("upstream must use http scheme".into());
    }
    let host = url.host_str().ok_or("invalid upstream host")?;
    let ip: IpAddr = host
        .parse()
        .map_err(|_| "upstream host must be numeric loopback IP")?;
    if !ip.is_loopback() {
        return Err("upstream must be loopback-only".into());
    }
    Ok(())
}
