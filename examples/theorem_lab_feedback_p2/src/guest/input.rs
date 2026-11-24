use crate::{Entry, MAX_INPUT, OP_SUBMIT, VERSION};
use alloc::collections::BTreeMap;
use alloc::string::String;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct In {
    pub v: u8,
    pub op: u8,
    #[serde(rename = "invokeId")]
    pub invoke_id: String,
    pub entry: Entry,
    #[serde(default)]
    pub fragments: BTreeMap<String, String>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
}
pub fn parse(input: &[u8]) -> Result<In, ()> {
    if input.len() > MAX_INPUT {
        return Err(());
    }
    let inp: In = serde_json::from_slice(input).map_err(|_| ())?;
    if inp.v != VERSION || inp.op != OP_SUBMIT {
        return Err(());
    }
    Ok(inp)
}
pub fn parse_hex32(s: &str) -> Result<[u8; 32], ()> {
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(());
    }
    let mut out = [0u8; 32];
    for (i, c) in out.iter_mut().enumerate() {
        *c = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| ())?;
    }
    Ok(out)
}
