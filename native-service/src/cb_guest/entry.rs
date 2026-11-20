use super::cap::{CbDisk, CbLock, CbRandom};
use super::fd;
use crate::cap::Eval;
use crate::reg::Registry;
struct NoEval;
impl Eval for NoEval {
    type Err = i32;
    fn eval(
        &self,
        _: &[u8],
        _: &[u8],
        _: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        Err(-13)
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let req = match fd::read_all(input_fd) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let disk = CbDisk;
    let lock = CbLock;
    let rng = CbRandom;
    let eval = NoEval;
    let reg = match Registry::open(&disk, "programs") {
        Ok(r) => r,
        Err(_) => return -5,
    };
    let response = crate::handle::handle(&reg, &disk, &lock, &rng, &eval, req.as_slice());
    match fd::write_all(output_fd, &response) {
        Ok(()) => 0,
        Err(e) => e,
    }
}
