//! Wire handling for least-privilege sender hint list/read/ack.
use super::res::{map_res, ok};
use super::{Disk, MutLock};
use crate::reg::Registry;
use crate::wire::Request;

pub fn handle<D: Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: Request,
) -> Option<Vec<u8>> {
    Some(match req {
        Request::HintList(cap, ns, key) => {
            map_res(reg.hint_list(disk, lock, &cap, &ns, &key).map(ok), 7)
        }
        Request::HintRead(cap, ns, key, seq) => {
            map_res(reg.hint_read(disk, lock, &cap, &ns, &key, seq).map(ok), 7)
        }
        Request::HintAck(cap, ns, key, seq) => map_res(
            reg.hint_ack(disk, lock, &cap, &ns, &key, seq)
                .map(|_| ok(Vec::new())),
            7,
        ),
        Request::SubFetch(cap, ns, key) => {
            map_res(reg.sub_fetch(disk, lock, &cap, &ns, &key).map(ok), 7)
        }
        _ => return None,
    })
}
