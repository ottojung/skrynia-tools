//! Read-only OP27 response mapping; no endpoint bytes cross this boundary.
use super::res::{mk_err, ok};
use crate::MutLock;

pub fn handle<D: super::Disk, L: MutLock>(
    reg: &crate::reg::Registry,
    disk: &D,
    lock: &L,
    grant: Vec<u8>,
    key: Vec<u8>,
    subscriber: Vec<u8>,
    expected: Vec<u8>,
) -> Vec<u8> {
    match reg.push_sub_status(disk, lock, &grant, &key, &subscriber, &expected) {
        Ok(crate::registry::appdata::push::Status::Active(hash)) => {
            let mut body = vec![1u8];
            body.extend_from_slice(&hash);
            ok(body)
        }
        Ok(crate::registry::appdata::push::Status::Absent) => ok(vec![0]),
        Ok(crate::registry::appdata::push::Status::Mismatch) => mk_err(8),
        Ok(crate::registry::appdata::push::Status::Invalid) => mk_err(9),
        Err(crate::PushErr::Busy) => mk_err(11),
        Err(crate::PushErr::Disk(_)) | Err(crate::PushErr::Lock(_)) => mk_err(7),
        Err(_) => mk_err(9),
    }
}
