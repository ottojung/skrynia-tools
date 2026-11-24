#[cfg(target_arch = "wasm32")]
use super::{map, ops};
#[cfg(target_arch = "wasm32")]
use crate::wdec::decode_req;
#[cfg(target_arch = "wasm32")]
use crate::wire::Request;

#[cfg(target_arch = "wasm32")]
pub fn dispatch(invoke_id: &[u8], req: &[u8]) -> Result<Vec<u8>, i32> {
    let req = decode_req(req).map_err(|_| -1)?;
    let disk = crate::root_disk::HostDisk;
    let lock = super::super::lock::HostLock;
    let reg = crate::reg::Registry::open(&disk, "programs").map_err(|_| -5)?;
    let b = match req {
        Request::AppGet { read_cap, key } if read_cap.as_slice() == invoke_id => reg
            .app_get(&disk, &read_cap, &key)
            .map(map::ok)
            .unwrap_or_else(|_| map::err(8)),
        Request::AppPut {
            write_cap,
            key,
            expect,
            value,
        } if write_cap.as_slice() == invoke_id => {
            ops::put(&disk, &lock, &reg.dir, invoke_id, key, expect, value)
        }
        Request::AppDelete {
            write_cap,
            key,
            expect,
        } if write_cap.as_slice() == invoke_id => {
            ops::del(&disk, &lock, &reg.dir, invoke_id, key, expect)
        }
        Request::AppGet { .. } | Request::AppPut { .. } | Request::AppDelete { .. } => map::err(13),
        _ => map::err(12),
    };
    Ok(b)
}
