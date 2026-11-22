//! Subscription JSON is untrusted wire data from the browser path.
use super::{Error, Result};
use base64ct::{Base64UrlUnpadded, Encoding};

pub struct Subscription {
    pub endpoint: String,
    pub p256dh: Vec<u8>,
    pub auth: Vec<u8>,
}

impl Subscription {
    pub fn parse(raw: &[u8]) -> Result<Self> {
        let value: serde_json::Value =
            serde_json::from_slice(raw).map_err(|_| Error::Wire("json"))?;
        let endpoint = value
            .get("endpoint")
            .and_then(|v| v.as_str())
            .ok_or(Error::Wire("endpoint"))?;
        if endpoint.is_empty() || endpoint.len() > 2048 {
            return Err(Error::Quota);
        }
        let keys = value.get("keys").ok_or(Error::Wire("keys"))?;
        Ok(Self {
            endpoint: endpoint.to_owned(),
            p256dh: decode_key(keys, "p256dh", 65)?,
            auth: decode_key(keys, "auth", 16)?,
        })
    }
}

fn decode_key(keys: &serde_json::Value, name: &str, expect: usize) -> Result<Vec<u8>> {
    let text = keys
        .get(name)
        .and_then(|v| v.as_str())
        .ok_or(Error::Wire("key"))?;
    let bytes = Base64UrlUnpadded::decode_vec(text).map_err(|_| Error::Wire("key-b64"))?;
    if bytes.len() != expect || (name == "p256dh" && bytes[0] != 4) {
        return Err(Error::Wire("key-len"));
    }
    Ok(bytes)
}
