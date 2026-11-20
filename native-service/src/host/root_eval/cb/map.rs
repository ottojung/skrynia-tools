#[cfg(target_arch = "wasm32")]
use crate::wenc::encode_res;
#[cfg(target_arch = "wasm32")]
use crate::wire::{Response, STATUS_ERR, STATUS_OK};

#[cfg(target_arch = "wasm32")]
pub fn map_app_err<D, L>(e: crate::AppErr<D, L>) -> Vec<u8> {
    use crate::AppErr;
    match e {
        AppErr::Conflict(c) => conflict(c),
        AppErr::Busy => err(11),
        AppErr::BadAuth | AppErr::BadKey => err(9),
        AppErr::NotFound => err(8),
        _ => err(7),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn ok(body: Vec<u8>) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_OK,
        body,
    })
}

#[cfg(target_arch = "wasm32")]
pub fn err(code: u8) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![code],
    })
}

#[cfg(target_arch = "wasm32")]
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
