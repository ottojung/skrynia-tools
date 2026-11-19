use crate::cases::support::{FsDisk, StdLock};
use skrynia_program0::cap::{Eval, Random};
use skrynia_program0::handle::handle;
use skrynia_program0::reg::Registry;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::Request;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Noop;
impl Eval for Noop {
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
impl Random for Noop {
    type Err = ();
    fn fill(&self, _: &mut [u8]) -> Result<(), ()> {
        Ok(())
    }
}

static N: AtomicU64 = AtomicU64::new(0);
pub fn tmp() -> PathBuf {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = N.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("skrynia-p0-appdata-{t}-{n}"))
}

pub fn call(
    reg: &Registry,
    disk: &FsDisk,
    lock: &StdLock,
    req: Request,
) -> skrynia_program0::wire::Response {
    decode_res(&handle(reg, disk, lock, &Noop, &Noop, &encode_req(&req))).unwrap()
}
