use super::res::{map_app_unit, map_app_write};
use crate::MutLock;
use crate::reg::Registry;
pub fn handle<D: super::Disk, L: MutLock>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    req: crate::wire::Request,
) -> Option<Vec<u8>> {
    use crate::wire::Request;
    Some(match req {
        Request::AppPutWake(w, k, e, v, g, wk) => map_app_write(
            reg.app_put_wake(disk, lock, &w, &k, &e, &v, &g, &wk)
                .map(|h| h.to_vec()),
        ),
        Request::AppDeleteWake(w, k, e, g, wk) => {
            map_app_unit(reg.app_delete_wake(disk, lock, &w, &k, &e, &g, &wk))
        }
        _ => return None,
    })
}
