use crate::wenc::encode_res;
use crate::wire::{Response, STATUS_ERR, STATUS_OK};

pub fn ok(body: Vec<u8>) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_OK,
        body,
    })
}

pub fn err(code: u8) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![code],
    })
}

pub fn conflict(cur: Option<[u8; 32]>) -> Vec<u8> {
    let mut body = vec![10u8];
    match cur {
        None => body.push(0),
        Some(h) => {
            body.push(1);
            body.extend_from_slice(&h);
        }
    }
    encode_res(&Response {
        status: STATUS_ERR,
        body,
    })
}
