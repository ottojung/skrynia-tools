use crate::cases::support::{MemDisk, NoLock, Rng};
use skrynia_program0::handle::handle;
use skrynia_program0::reg::Registry;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::{Request, Response};
pub struct Fix;
pub fn setup() -> (Registry, MemDisk, NoLock) {
    let d = MemDisk::default();
    (Registry::open(&d, "p").unwrap(), d, NoLock)
}
pub fn call(r: &Registry, d: &MemDisk, l: &NoLock, req: Request) -> Response {
    decode_res(&handle(r, d, l, &Rng(0), &Fix, &encode_req(&req))).unwrap()
}
impl skrynia_program0::cap::Eval for Fix {
    type Err = ();
    fn eval(
        &self,
        _: &[u8],
        _: &[u8],
        _: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok(vec![])
    }
}
impl skrynia_program0::cap::Random for Fix {
    type Err = ();
    fn fill(&self, _: &mut [u8]) -> Result<(), ()> {
        Ok(())
    }
}
