use anyhow::{Result, bail};
const HEADER: &[u8] = b"skrynia-log-entry-v1\0";

pub fn encode(id: &str, data: &[u8]) -> Result<Vec<u8>> {
    if id.len() != 32 || !id.bytes().all(|c| c.is_ascii_hexdigit()) {
        bail!("operation ID must be exactly 32 ASCII hex characters");
    }
    let mut bytes = HEADER.to_vec();
    bytes.extend_from_slice(id.as_bytes());
    bytes.extend_from_slice(data);
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<(&str, &[u8])> {
    let Some(rest) = bytes.strip_prefix(HEADER) else {
        bail!("invalid log entry");
    };
    if rest.len() < 32 {
        bail!("truncated log entry");
    }
    let (id, payload) = rest.split_at(32);
    let id = std::str::from_utf8(id)?;
    if !id.bytes().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid operation ID");
    }
    Ok((id, payload))
}
