mod errors;
mod mutate;
mod read;
mod wire;
use skrynia_tools::wire::v2compat::Request;
use skrynia_v2_provider::{Callback, InvokeError};
use std::collections::BTreeMap;

pub struct Fake;
impl Callback for Fake {
    fn invoke(&self, bytes: &[u8]) -> Result<Vec<u8>, InvokeError> {
        if bytes.len() < 3 || bytes[0] != 1 {
            return Err(InvokeError::Unavailable);
        }
        let len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
        if bytes.len() != 3 + len {
            return Err(InvokeError::Unavailable);
        }
        match &bytes[3..] {
            b"public/file.txt" => Ok([vec![1], b"abcdef".to_vec()].concat()),
            b"public/missing.txt" => Ok(vec![0]),
            _ => Err(InvokeError::Unavailable),
        }
    }
}
pub fn req(method: &str, path: &str) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: BTreeMap::new(),
        body: Vec::new(),
    }
}
