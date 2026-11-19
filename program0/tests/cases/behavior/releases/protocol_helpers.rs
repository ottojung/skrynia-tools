use crate::cases::support::{MemDisk, NoLock, Rng};
use skrynia_program0::{cap::Eval, handle::handle, reg::Registry};
use skrynia_program0::{
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, Response},
};
struct EvalTest;
impl Eval for EvalTest {
    type Err = ();
    fn eval(
        &self,
        code: &[u8],
        _: &[u8],
        _: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok(code.to_vec())
    }
}
pub fn call(r: &Registry, d: &MemDisk, l: &NoLock, req: Request) -> Response {
    decode_res(&handle(r, d, l, &Rng(0), &EvalTest, &encode_req(&req))).unwrap()
}
pub fn hashes(body: &[u8]) -> Vec<[u8; 32]> {
    assert!(body.len() >= 2);
    let len = u16::from_le_bytes(body[0..2].try_into().unwrap()) as usize;
    assert_eq!(body.len(), 2 + len * 32);
    body[2..]
        .chunks_exact(32)
        .map(|x| x.try_into().unwrap())
        .collect()
}
