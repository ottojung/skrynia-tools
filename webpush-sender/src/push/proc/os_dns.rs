use super::{IoFailure, Resolver};
use std::net::{IpAddr, ToSocketAddrs};

pub struct OsResolver;

impl Resolver for OsResolver {
    fn resolve(&self, host: &str) -> core::result::Result<Vec<IpAddr>, IoFailure> {
        let addrs = (host, 443u16).to_socket_addrs().map_err(|_| IoFailure)?;
        let out: Vec<IpAddr> = addrs.map(|a| a.ip()).collect();
        if out.is_empty() {
            return Err(IoFailure);
        }
        Ok(out)
    }
}
