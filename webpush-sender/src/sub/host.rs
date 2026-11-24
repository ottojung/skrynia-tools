use super::{Error, Result};

pub struct Authority {
    pub host: String,
    pub port: u16,
}

pub fn parse(authority: &str) -> Result<Authority> {
    if authority.is_empty() || authority.contains('@') || authority.contains('[') {
        return Err(Error::Ssrf("authority"));
    }
    let (host, port) = authority.rsplit_once(':').unwrap_or((authority, "443"));
    if port != "443" {
        return Err(Error::Ssrf("port"));
    }
    if host.is_empty() || host.len() > 253 || host.starts_with('.') || host.ends_with('.') {
        return Err(Error::Ssrf("host"));
    }
    if !host
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
    {
        return Err(Error::Ssrf("host-chars"));
    }
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Err(Error::Ssrf("ip-literal"));
    }
    Ok(Authority {
        host: host.to_owned(),
        port: 443,
    })
}
