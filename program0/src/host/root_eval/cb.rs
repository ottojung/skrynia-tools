#[cfg(target_arch = "wasm32")]
use crate::wdec::decode_req;
#[cfg(target_arch = "wasm32")]
use crate::wenc::encode_res;
#[cfg(target_arch = "wasm32")]
use crate::wire::{Request, Response, STATUS_ERR, STATUS_OK};

#[cfg(target_arch = "wasm32")]
pub(super) static mut CB_APP: Option<Vec<u8>> = None;

#[cfg(target_arch = "wasm32")]
pub fn dispatch(app: &[u8], req: &[u8]) -> Result<Vec<u8>, i32> {
    let req = decode_req(req).map_err(|_| -1)?;
    let disk = crate::root_disk::HostDisk;
    let reg = crate::reg::Registry::open(&disk, "programs").map_err(|_| -5)?;
    let b = match req {
        Request::AppGet { app: a, key } if a == app => reg
            .app_get(&disk, &a, &key)
            .map(|b| ok(b))
            .unwrap_or_else(|_| err(8)),
        Request::AppPut { app: a, key, value } if a == app => reg
            .app_put(&disk, &a, &key, &value)
            .map(|_| ok(Vec::new()))
            .unwrap_or_else(|_| err(7)),
        Request::AppGet { .. } | Request::AppPut { .. } => err(13),
        _ => err(12),
    };
    Ok(b)
}
#[cfg(target_arch = "wasm32")]
fn ok(body: Vec<u8>) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_OK,
        body,
    })
}
#[cfg(target_arch = "wasm32")]
fn err(code: u8) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![code],
    })
}
