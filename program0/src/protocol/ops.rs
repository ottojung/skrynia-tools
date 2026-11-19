use super::cap::{Disk, Eval, Random};
use super::res::{err, ok};
use crate::reg::Registry;
use crate::wire::Request;

pub fn dispatch<D: Disk, R: Random, E: Eval>(
    reg: &Registry,
    disk: &D,
    rng: &R,
    eval: &E,
    req: Request,
) -> Vec<u8> {
    match req {
        Request::Register { code } => reg
            .register(disk, rng, code)
            .map(ok)
            .unwrap_or_else(|_| err(2)),
        Request::Set { id, code } => reg
            .set(disk, &id, code)
            .map(|_| ok(Vec::new()))
            .unwrap_or_else(|_| err(3)),
        Request::Invoke { id, input } => reg
            .invoke(disk, eval, &id, &input)
            .map(ok)
            .unwrap_or_else(|_| err(4)),
        Request::Lookup { id } => reg.lookup(&id).map(ok).unwrap_or_else(|_| err(5)),
        Request::Revoke { id } => reg
            .revoke(disk, &id)
            .map(|_| ok(Vec::new()))
            .unwrap_or_else(|_| err(6)),
        Request::AppPut { app, key, value } => reg
            .app_put(disk, &app, &key, &value)
            .map(|_| ok(Vec::new()))
            .unwrap_or_else(|_| err(7)),
        Request::AppGet { app, key } => reg
            .app_get(disk, &app, &key)
            .map(ok)
            .unwrap_or_else(|_| err(8)),
    }
}
