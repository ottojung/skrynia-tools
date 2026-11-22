use super::res::ok as resp_ok;
use crate::MutLock;
use crate::PushErr;
use crate::reg::Registry;

pub fn handle<D: super::Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: crate::wire::Request,
) -> Option<Vec<u8>> {
    use crate::wire::Request;
    Some(match req {
        Request::PushSubscribe(write_cap, key, subscriber, subscription) => {
            match reg.push_subscribe(disk, lock, &write_cap, &key, &subscriber, &subscription) {
                Ok(()) => resp_ok(Vec::new()),
                Err(e) => push_err(e),
            }
        }
        Request::PushUnsubscribe(write_cap, key, subscriber) => {
            match reg.push_unsubscribe(disk, lock, &write_cap, &key, &subscriber) {
                Ok(()) => resp_ok(Vec::new()),
                Err(e) => push_err(e),
            }
        }
        _ => return None,
    })
}

fn push_err<D, L>(e: PushErr<D, L>) -> Vec<u8> {
    use super::res::mk_err;
    match e {
        PushErr::BadAuth | PushErr::BadKey => mk_err(9),
        PushErr::Missing => mk_err(8),
        PushErr::Busy => mk_err(11),
        PushErr::Quota => mk_err(10),
        _ => mk_err(7),
    }
}
