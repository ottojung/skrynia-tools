use super::{Error, Result, Target, VapidKey, authorization};
use std::net::IpAddr;

#[derive(Clone)]
pub struct PushRequest {
    pub host: String,
    pub ip: IpAddr,
    pub port: u16,
    pub path: String,
    pub authorization: String,
    pub ttl: u32,
    pub topic: [u8; 16],
    pub body: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    target: &Target,
    body: Vec<u8>,
    key: &VapidKey,
    exp: u64,
    sub: &str,
    ttl: u32,
    topic: [u8; 16],
) -> Result<PushRequest> {
    if body.is_empty() || body.len() > 4096 + 130 {
        return Err(Error::Quota);
    }
    // RFC 6454 §6.1 step 5 / RFC 8292 §2: the default HTTPS port is omitted
    // from the origin serialization; a real push service rejects "host:443".
    let aud = if target.port == 443 {
        format!("https://{}", target.host)
    } else {
        format!("https://{}:{}", target.host, target.port)
    };
    let jwt = key.sign(&aud, exp, sub)?;
    Ok(PushRequest {
        host: target.host.clone(),
        ip: target.ip,
        port: target.port,
        path: target.path.clone(),
        authorization: authorization(key, &jwt),
        ttl,
        topic,
        body,
    })
}
