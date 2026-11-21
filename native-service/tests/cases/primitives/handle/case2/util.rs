use super::{CatEval, MemDisk, NoLock, Registry, Rng, decode_res, encode_req, handle};
use skrynia_native_service::wire::{Request, Response};

pub fn call(reg: &Registry, disk: &MemDisk, lock: &NoLock, req: Request) -> Response {
    decode_res(&handle(
        reg,
        disk,
        lock,
        &Rng(2),
        &CatEval,
        &encode_req(&req),
    ))
    .unwrap()
}
