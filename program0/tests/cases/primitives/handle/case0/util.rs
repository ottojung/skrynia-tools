use super::{CatEval, MemDisk, NoLock, Registry, Rng, decode_res, encode_req, handle};
use skrynia_program0::wire::{Request, Response};

pub fn call(reg: &Registry, disk: &MemDisk, lock: &NoLock, req: Request) -> Response {
    decode_res(&handle(
        reg,
        disk,
        lock,
        &Rng(9),
        &CatEval,
        &encode_req(&req),
    ))
    .unwrap()
}
