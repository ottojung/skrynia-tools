#[link(wasm_import_module = "skrynia")]
unsafe extern "C" {
    fn invoke(rp: u32, rl: u32, dp: u32, dc: u32) -> i32;
}

pub fn raw(req: &[u8], cap: usize) -> Result<Vec<u8>, i32> {
    let mut out = vec![0u8; cap.max(8)];
    loop {
        let n = unsafe {
            invoke(
                req.as_ptr() as u32,
                req.len() as u32,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        };
        if n == -11 && out.len() < 1024 * 1024 {
            out.resize((out.len() * 2).min(1024 * 1024), 0);
            continue;
        }
        if n < 0 {
            return Err(n);
        }
        out.truncate(n as usize);
        return Ok(out);
    }
}
pub fn call(req: &[u8]) -> Result<(i32, Vec<u8>), i32> {
    call_with_capacity(req, 256)
}

pub fn call_with_capacity(req: &[u8], capacity: usize) -> Result<(i32, Vec<u8>), i32> {
    let res = raw(req, capacity)?;
    if res.len() < 4 {
        return Err(-1);
    }
    let code = super::wire::i32_at(&res[0..4]);
    Ok((code, res[4..].to_vec()))
}
