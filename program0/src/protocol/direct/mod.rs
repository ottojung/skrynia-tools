mod change;
mod create;
mod exec;
mod read;
use super::{
    cap::{Disk, Eval, Random},
    res::{err, ok},
};
use super::{wdec::decode_req, wire::Request};
#[allow(unused_imports)]
use crate::store_io;

pub fn handle<D: Disk, R: Random, E: Eval>(disk: &D, rng: &R, eval: &E, req: &[u8]) -> Vec<u8> {
    let req = match decode_req(req) {
        Ok(value) => value,
        Err(_) => return err(1),
    };
    match req {
        Request::Register { code } => create::register(disk, rng, &code),
        Request::Set { id, code } => change::set(disk, &id, &code),
        Request::Revoke { id } => change::revoke(disk, &id),
        Request::Lookup { id } => read::lookup(disk, &id),
        Request::Invoke { id, input } => exec::invoke(disk, eval, &id, &input),
        Request::V2Bootstrap { .. } | Request::V2Mutate { .. } => err(7),
    }
}
