//! Wire handling for exact-key push, explicit grants and scoped sender hints.
use super::res::{map_res, ok};
use crate::MutLock;
use crate::reg::Registry;
use crate::wire::Request;

pub fn handle<D: super::Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: Request,
) -> Option<Vec<u8>> {
    if matches!(
        &req,
        Request::HintList { .. }
            | Request::HintRead { .. }
            | Request::HintAck { .. }
            | Request::SubFetch { .. }
    ) {
        return super::sender::handle(reg, disk, lock, req);
    }
    Some(match req {
        Request::PushSubscribe(grant_cap, key, subscriber, subscription) => map_res(
            reg.push_subscribe(disk, lock, &grant_cap, &key, &subscriber, &subscription)
                .map(|_| ok(Vec::new())),
            7,
        ),
        Request::PushUnsubscribe(grant_cap, key, subscriber) => map_res(
            reg.push_unsubscribe(disk, lock, &grant_cap, &key, &subscriber)
                .map(|_| ok(Vec::new())),
            7,
        ),
        Request::PushGrant(write_cap, grant_cap, key, subscriber) => map_res(
            reg.push_grant(disk, lock, &write_cap, &grant_cap, &key, &subscriber)
                .map(|_| ok(Vec::new())),
            7,
        ),
        Request::PushSenderGrant(write_cap, sender_cap, key) => map_res(
            reg.push_sender_grant(disk, lock, &write_cap, &sender_cap, &key)
                .map(|_| ok(Vec::new())),
            7,
        ),
        _ => return None,
    })
}
