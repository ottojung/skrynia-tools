mod case0;
mod case1;
mod case2;
use crate::cases::support::{MemDisk, NoLock, Rng};
use skrynia_native_service::auth;
use skrynia_native_service::cap::Eval;
use skrynia_native_service::handle::handle;
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wdec::decode_res;
use skrynia_native_service::wenc::encode_req;
use skrynia_native_service::wire::{AppPrecond, Request, STATUS_ERR, STATUS_OK};

struct CatEval;
impl Eval for CatEval {
    type Err = ();
    fn eval(
        &self,
        w: &[u8],
        i: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok([w, b":", i].concat())
    }
}
