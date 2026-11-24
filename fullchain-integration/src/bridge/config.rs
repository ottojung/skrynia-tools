//! Fail-closed config for the external-edge sender bridge.
use std::path::PathBuf;

pub struct Config {
    pub edge_port: u16,
    pub edge_host: String,
    pub edge_origin: String,
    pub edge_cap: String,
    pub sender_cap: [u8; 32],
    pub ns: [u8; 32],
    pub key: Vec<u8>,
    pub ledger: PathBuf,
    pub out: PathBuf,
}

fn hex32(v: &serde_json::Value, name: &str) -> [u8; 32] {
    let s = v.get(name).and_then(|x| x.as_str()).expect(name);
    assert_eq!(s.len(), 64, "{name} must be 64 hex chars");
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect(name);
    }
    out
}

pub fn load() -> Config {
    let path = std::env::var("FC_SENDER_BRIDGE").expect("FC_SENDER_BRIDGE config path required");
    let raw = std::fs::read_to_string(&path).expect("bridge config readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("bridge config JSON");
    let str_field = |name: &str| -> String {
        v.get(name).and_then(|x| x.as_str()).expect(name).to_owned()
    };
    let key = str_field("key").into_bytes();
    assert!(!key.is_empty() && key.len() <= 255, "key length");
    Config {
        edge_port: v.get("edge_port").and_then(|x| x.as_u64()).expect("edge_port") as u16,
        edge_host: str_field("edge_host"),
        edge_origin: str_field("edge_origin"),
        edge_cap: str_field("edge_cap"),
        sender_cap: hex32(&v, "sender_cap"),
        ns: hex32(&v, "ns"),
        key,
        ledger: PathBuf::from(str_field("ledger")),
        out: PathBuf::from(str_field("out")),
    }
}
