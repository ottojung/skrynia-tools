#[cfg(target_arch = "wasm32")]
use crate::cap::Disk;
#[cfg(target_arch = "wasm32")]
use crate::guest_sys::{call, proto};

#[cfg(target_arch = "wasm32")]
pub struct CbDisk;

#[cfg(target_arch = "wasm32")]
impl Disk for CbDisk {
    type Err = i32;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, i32> {
        let mut req = vec![proto::DISK_READ];
        proto::put_bytes(&mut req, path.as_bytes());
        let res = call::check_ok(call::call(&req)?)?;
        match res.first().copied() {
            Some(proto::NONE) => Ok(None),
            Some(proto::SOME) => {
                let mut i = 1usize;
                Ok(Some(proto::get_bytes(&res, &mut i).ok_or(-1)?))
            }
            _ => Err(-1),
        }
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), i32> {
        let mut req = vec![proto::DISK_WRITE_ATOMIC];
        proto::put_bytes(&mut req, path.as_bytes());
        proto::put_bytes(&mut req, bytes);
        call::check_ok(call::call(&req)?).map(|_| ())
    }
    fn remove(&self, path: &str) -> Result<(), i32> {
        let mut req = vec![proto::DISK_REMOVE];
        proto::put_bytes(&mut req, path.as_bytes());
        call::check_ok(call::call(&req)?).map(|_| ())
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, i32> {
        let mut req = vec![proto::DISK_LIST];
        proto::put_bytes(&mut req, dir.as_bytes());
        let res = call::check_ok(call::call(&req)?)?;
        let mut i = 0usize;
        let n = proto::get_u32(&res, &mut i).ok_or(-1)? as usize;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(proto::get_str(&res, &mut i).ok_or(-1)?);
        }
        Ok(out)
    }
}
