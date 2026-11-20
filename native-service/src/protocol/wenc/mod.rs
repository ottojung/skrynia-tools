pub use super::wenc_req::encode_req;
use crate::wire::Response;

pub fn encode_res(res: &Response) -> Vec<u8> {
    let mut out = vec![crate::wire::VERSION, res.status];
    out.extend_from_slice(&(res.body.len() as u32).to_le_bytes());
    out.extend_from_slice(&res.body);
    out
}
