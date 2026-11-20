use super::{GuestEval, MemDisk, NoLock, Registry, Rng, decode_res, encode_req, handle};
use skrynia_native_service::wire::Request;

pub fn call(
    r: &Registry,
    d: &MemDisk,
    l: &NoLock,
    req: Request,
) -> skrynia_native_service::wire::Response {
    let res = handle(r, d, l, &Rng(0), &GuestEval, &encode_req(&req));
    decode_res(&res).unwrap()
}
