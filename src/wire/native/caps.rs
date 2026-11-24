use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Capabilities {
    pub read: [u8; 32],
    pub write: Option<[u8; 32]>,
}
impl Capabilities {
    pub fn new(read: &str, write: Option<&str>) -> Result<Self> {
        let read = parse_cap(read)?;
        let write = write.map(parse_cap).transpose()?;
        if write.is_some_and(|w| digest(&w) != read) {
            bail!("read and write capabilities do not match");
        }
        Ok(Self { read, write })
    }
    pub fn writer(&self) -> Result<&[u8; 32]> {
        self.write
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("write capability required"))
    }
}
pub fn parse_cap(hexadecimal: &str) -> Result<[u8; 32]> {
    let mut out = [0; 32];
    hex::decode_to_slice(hexadecimal, &mut out)
        .map_err(|_| anyhow::anyhow!("invalid 32-byte hexadecimal value"))?;
    Ok(out)
}
pub fn digest(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}
