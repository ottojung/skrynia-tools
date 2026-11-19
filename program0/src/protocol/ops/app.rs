use super::res::{map_app_get, map_app_unit, map_app_write, map_res, ok as resp_ok};
use crate::MutLock;
use crate::reg::Registry;

pub fn handle<D: super::Disk, L: MutLock, E: super::Eval>(
    reg: &Registry,
    disk: &D,
    lock: &L,
    eval: &E,
    req: crate::wire::Request,
) -> Option<Vec<u8>> {
    use crate::wire::Request;
    Some(match req {
        Request::AppGet { read_cap, key } => map_app_get(reg.app_get(disk, &read_cap, &key)),
        Request::AppPut {
            write_cap,
            key,
            expect,
            value,
        } => map_app_write(
            reg.app_put(disk, lock, &write_cap, &key, &expect, &value)
                .map(|h| h.to_vec()),
        ),
        Request::AppDelete {
            write_cap,
            key,
            expect,
        } => map_app_unit(reg.app_delete(disk, lock, &write_cap, &key, &expect)),
        Request::Invoke { invoke_id, input } => map_res(
            reg.invoke(disk, lock, eval, &invoke_id, &input)
                .map(resp_ok),
            4,
        ),
        Request::Lookup { invoke_id } => map_res(reg.lookup(disk, &invoke_id).map(resp_ok), 5),
        _ => return None,
    })
}
