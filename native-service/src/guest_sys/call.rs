#[cfg(target_arch = "wasm32")]
use super::proto;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn invoke(req_ptr: u32, req_len: u32, res_ptr: u32, res_cap: u32) -> i32;
}

#[cfg(target_arch = "wasm32")]
pub fn call(req: &[u8]) -> Result<Vec<u8>, i32> {
    let mut out = vec![0u8; 1024 * 1024];
    loop {
        let n = unsafe {
            invoke(
                req.as_ptr() as u32,
                req.len() as u32,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        };
        if n == -10 {
            let new = out.len().saturating_mul(2).min(8 * 1024 * 1024);
            if new == out.len() {
                return Err(-10);
            }
            out.resize(new, 0);
            continue;
        }
        if n < 0 {
            return Err(n);
        }
        out.truncate(n as usize);
        return Ok(out);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn check_ok(res: Vec<u8>) -> Result<Vec<u8>, i32> {
    match res.first().copied() {
        Some(proto::OK) => Ok(res[1..].to_vec()),
        Some(proto::ERR) => {
            let mut i = 1usize;
            Err(proto::get_u32(&res, &mut i).map(|e| e as i32).unwrap_or(-1))
        }
        _ => Err(-1),
    }
}
