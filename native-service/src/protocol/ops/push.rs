use super::res::{map_res, ok as resp_ok};
use crate::MutLock;
use crate::reg::Registry;
use crate::PushErr;

pub fn handle<D: super::Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: crate::wire::Request,
) -> Option<Vec<u8>> {
    use crate::wire::Request;
    Some(match req {
        Request::PushSubscribe {
            write_cap,
            key,
            subscriber,
            subscription,
        } => match reg.push_subscribe(disk, lock, &write_cap, &key, &subscriber, &subscription) {
            Ok(()) => resp_ok(Vec::new()),
            Err(e) => push_err(e),
        },
        Request::PushUnsubscribe {
            write_cap,
            key,
            subscriber,
        } => match reg.push_unsubscribe(disk, lock, &write_cap, &key, &subscriber) {
            Ok(()) => resp_ok(Vec::new()),
            Err(e) => push_err(e),
        },
        _ => return None,
    })
}

fn push_err<D, L>(e: PushErr<D, L>) -> Vec<u8> {
    use super::res::mk_err;
    use crate::PushErr as P;
    match e {
        P::BadAuth | P::BadKey => mk_err(9),
        P::Missing => mk_err(8),
        P::Busy => mk_err(11),
        P::Quota => mk_err(10),
        _ => mk_err(7),
    }
}
