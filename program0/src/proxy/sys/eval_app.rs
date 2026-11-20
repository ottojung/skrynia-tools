use super::super::abi;
use super::eval::{APP_CTX, skrynia_host_cb};
use super::proto;
pub fn run(req: &[u8]) -> Vec<u8> {
    let mut i = 1usize;
    let (Some(code), Some(input), Some(iid)) = (
        proto::get_bytes(req, &mut i),
        proto::get_bytes(req, &mut i),
        proto::get_bytes(req, &mut i),
    ) else {
        return proto::err(-1);
    };
    unsafe { APP_CTX = Some(iid) };
    let cb_idx = skrynia_host_cb as *const () as usize as i32;
    let mut out = vec![0u8; 1024 * 1024];
    let mut cap = out.len();
    let n = loop {
        let n = unsafe {
            abi::eval(
                code.as_ptr() as u32,
                code.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                cb_idx,
                out.as_mut_ptr() as u32,
                cap as u32,
            )
        };
        if n == -10 && cap < 8 * 1024 * 1024 {
            cap = (cap * 2).min(8 * 1024 * 1024);
            out.resize(cap, 0);
            continue;
        }
        break n;
    };
    unsafe { APP_CTX = None };
    if n < 0 {
        return proto::err(n);
    }
    out.truncate(n as usize);
    let mut r = vec![proto::OK];
    proto::put_bytes(&mut r, &out);
    r
}
