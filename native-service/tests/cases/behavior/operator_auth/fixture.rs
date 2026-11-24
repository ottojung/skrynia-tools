use super::{MemDisk, NoLock, Rng};
use skrynia_native_service::{
    cap::Eval,
    handle::handle,
    reg::Registry,
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, Response},
};
struct NoEval;
impl Eval for NoEval {
    type Err = ();
    fn eval(
        &self,
        _: &[u8],
        _: &[u8],
        _: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Err(())
    }
}
pub fn call(disk: &MemDisk, req: Request) -> Response {
    let reg = Registry::open(disk, "reg").unwrap();
    decode_res(&handle(
        &reg,
        disk,
        &NoLock,
        &Rng(5),
        &NoEval,
        &encode_req(&req),
    ))
    .unwrap()
}
pub fn install(admin: &[u8]) -> Request {
    Request::Register {
        operator_cap: vec![85u8; 32],
        admin_cap: admin.to_vec(),
        code: b"test".to_vec(),
    }
}
