use super::super::{dispatch, open};
use skrynia_native_service::MutLock;
use skrynia_native_service::cap::{Disk, Eval, Random};
use skrynia_native_service::wdec::decode_res;
use skrynia_native_service::wenc::encode_req;
use skrynia_native_service::wire::{Request, STATUS_ERR, STATUS_OK};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct Mem(Mutex<BTreeMap<String, Vec<u8>>>);
impl Disk for Mem {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.lock().unwrap().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.lock().unwrap().insert(p.into(), b.to_vec());
        Ok(())
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.0.lock().unwrap().remove(p);
        Ok(())
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        let p = format!("{}/", d.trim_end_matches('/'));
        Ok(self
            .0
            .lock()
            .unwrap()
            .keys()
            .filter_map(|k| k.strip_prefix(&p).map(str::to_string))
            .collect())
    }
}
pub struct L;
impl MutLock for L {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        Ok(true)
    }
    fn release(&self) -> Result<(), ()> {
        Ok(())
    }
}
pub struct R;
impl Random for R {
    type Err = ();
    fn fill(&self, _: &mut [u8]) -> Result<(), ()> {
        Ok(())
    }
}
pub struct E;
impl Eval for E {
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
pub fn open_p(d: &Mem) -> skrynia_native_service::reg::Registry {
    d.write_atomic("p/operator.cap", &[85u8; 32]).unwrap();
    open(d, "p").unwrap()
}
pub fn ok(reg: &skrynia_native_service::reg::Registry, d: &Mem, req: Request) -> Vec<u8> {
    let r = decode_res(&dispatch(reg, d, &L, &R, &E, &encode_req(&req))).unwrap();
    assert_eq!(r.status, STATUS_OK);
    r.body
}
pub fn err(reg: &skrynia_native_service::reg::Registry, d: &Mem, req: Request) -> u8 {
    let r = decode_res(&dispatch(reg, d, &L, &R, &E, &encode_req(&req))).unwrap();
    assert_eq!(r.status, STATUS_ERR);
    r.body[0]
}
