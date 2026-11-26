//! Target entrypoint: a nested guest exports `_start () -> ()` and imports
//! exactly one evaluator-supplied function. The application wire protocol,
//! authority and registry logic below this file are unchanged.

use super::cap::{CbDisk, CbLock, CbRandom};
use super::eval::P1CallbackEval;
use super::fd;
use crate::reg::Registry;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    if run() != 0 {
        core::arch::wasm32::unreachable();
    }
}

fn run() -> i32 {
    let req = match fd::read_request() {
        Ok(r) => r,
        Err(e) => return e,
    };
    // K10 root tokens select the confined namespace at the kernel ABI. All
    // registry writes, reads, lists and fsync parents live under user/.
    let reg = match Registry::open(&CbDisk, "user/programs") {
        Ok(r) => r,
        Err(_) => return -5,
    };
    let response = crate::handle::handle(
        &reg,
        &CbDisk,
        &CbLock,
        &CbRandom,
        &P1CallbackEval,
        req.as_slice(),
    );
    match fd::write_response(&response) {
        Ok(()) => 0,
        Err(e) => e,
    }
}
