use super::cap::Disk;
use super::ops;
use crate::wdec::decode_req;
use crate::wenc::encode_res;
use crate::wire::{Request, Response, STATUS_ERR, STATUS_OK};

pub fn guest_callback<D: Disk>(
    disk: &D,
    dir: &str,
    app: &[u8],
    req: &[u8],
) -> Result<Vec<u8>, i32> {
    let req = decode_req(req).map_err(|_| -1)?;
    let bytes = match req {
        Request::AppGet { app: a, key } if a == app => resp(ops::get(disk, dir, &a, &key), 8),
        Request::AppPut { app: a, key, value } if a == app => {
            resp(ops::put(disk, dir, &a, &key, &value).map(|_| Vec::new()), 7)
        }
        Request::AppGet { .. } | Request::AppPut { .. } => err(13),
        _ => err(12),
    };
    Ok(bytes)
}

fn resp<E>(r: Result<Vec<u8>, E>, code: u8) -> Vec<u8> {
    match r {
        Ok(b) => ok(b),
        Err(_) => err(code),
    }
}
fn ok(body: Vec<u8>) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_OK,
        body,
    })
}
fn err(code: u8) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![code],
    })
}
