//! Target-ABI child transport: the only import is `p0.invoke`.
//! Framed requests are written one byte at a time with PUT, committed once
//! with FLUSH, and replies are drained with GET. A failing FLUSH is final.

#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}

pub const GET: i32 = 0;
pub const PUT: i32 = 1;
pub const FLUSH: i32 = 2;
pub const EOF: i64 = -1;

pub fn get() -> i64 {
    unsafe { invoke(0, GET, 0) }
}

fn put(byte: u8) -> bool {
    unsafe { invoke(0, PUT, i32::from(byte)) >= 0 }
}

fn flush() -> bool {
    unsafe { invoke(0, FLUSH, 0) >= 0 }
}

/// Execute one complete framed request and drain its reply.
pub fn call(req: &[u8], out: &mut [u8]) -> i32 {
    for &byte in req {
        if !put(byte) {
            return -3;
        }
    }
    if !flush() {
        return -5;
    }
    let mut len = 0usize;
    loop {
        match get() {
            EOF => return len as i32,
            code if code < 0 => return code as i32,
            code if len >= out.len() => return -6,
            code => {
                out[len] = code as u8;
                len += 1;
            }
        }
    }
}
