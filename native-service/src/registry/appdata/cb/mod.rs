mod map;
mod resp;
use super::cap::Disk;
use super::ops::{self, AppErr, WriteAuth};
use crate::MutLock;
use crate::wdec::decode_req;
use crate::wire::Request;
use map::{map_read, map_unit, map_write};
use resp::err;

pub fn guest_callback<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    dir: &str,
    invoke_id: &[u8],
    req: &[u8],
) -> Result<Vec<u8>, i32> {
    let req = decode_req(req).map_err(|_| -1)?;
    let bytes = match req {
        Request::AppGet { read_cap, key } if read_cap.as_slice() == invoke_id => {
            map_read(ops::get(disk, dir, &read_cap, &key))
        }
        Request::AppPut {
            write_cap,
            key,
            expect,
            value,
        } if write_cap.as_slice() == invoke_id => {
            let auth = WriteAuth::SelfNs(invoke_id);
            map_write(ops::put(disk, lock, dir, auth, &key, &expect, &value))
        }
        Request::AppDelete {
            write_cap,
            key,
            expect,
        } if write_cap.as_slice() == invoke_id => {
            let auth = WriteAuth::SelfNs(invoke_id);
            map_unit(ops::delete(disk, lock, dir, auth, &key, &expect))
        }
        Request::AppGet { .. } | Request::AppPut { .. } | Request::AppDelete { .. } => err(13),
        _ => err(12),
    };
    Ok(bytes)
}
