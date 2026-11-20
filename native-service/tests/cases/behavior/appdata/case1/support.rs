use crate::cases::support::{FsDisk, StdLock};
use skrynia_native_service::cap::{Eval, Random};
use skrynia_native_service::handle::handle;
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wdec::decode_res;
use skrynia_native_service::wenc::encode_req;
use skrynia_native_service::wire::Request;
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
) -> skrynia_native_service::wire::Response {
    decode_res(&handle(reg, disk, lock, &Noop, &Noop, &encode_req(&req))).unwrap()
}
