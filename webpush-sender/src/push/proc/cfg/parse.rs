use super::RuntimeConfig;
use super::field::{str_field, u16_field, u64_field, usize_field};
use super::hex::hex32;
use super::pump::parse_pump;
use super::{Edge, Error, Result};
use serde_json::Value;
use std::net::IpAddr;
use std::path::PathBuf;

pub(super) fn parse(v: &Value) -> Result<RuntimeConfig> {
    let edge = parse_edge(v.get("edge").ok_or(Error::Config("config-edge"))?)?;
    let pump = parse_pump(v)?;
    let vapid_private_key = hex32(v, "vapid_private_key")?;
    let ledger_dir = PathBuf::from(str_field(v, "ledger_dir")?);
    let poll_ms = u64_field(v, "poll_ms")?;
    if !(250..=300_000).contains(&poll_ms) {
        return Err(Error::Config("config-poll"));
    }
    Ok(RuntimeConfig {
        edge,
        pump,
        vapid_private_key,
        ledger_dir,
        poll_ms,
    })
}

fn parse_edge(v: &Value) -> Result<Edge> {
    let ip: IpAddr = str_field(v, "ip")?
        .parse()
        .map_err(|_| Error::Config("config-edge"))?;
    let edge = Edge {
        ip,
        port: u16_field(v, "port")?,
        host: str_field(v, "host")?.to_owned(),
        origin: str_field(v, "origin")?.to_owned(),
        capability: str_field(v, "capability")?.to_owned(),
        timeout_ms: u64_field(v, "timeout_ms")?,
        max_request_bytes: usize_field(v, "max_request_bytes")?,
        max_response_bytes: usize_field(v, "max_response_bytes")?,
    };
    edge.validate()?;
    Ok(edge)
}
