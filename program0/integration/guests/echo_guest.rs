//! Minimal `p0`/`invoke` child for disposable P0 validation: byte echo using
//! the T3-agreed GET/PUT/FLUSH ops and framed generic WRITE opcode.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}
const GET: i32 = 0;
const PUT: i32 = 1;
const FLUSH: i32 = 2;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let mut data = Vec::new();
    unsafe {
        loop {
            let b = invoke(0, GET, 0);
            if b < 0 || b > 255 { break }
            data.push(b as u8);
        }
        let mut frame = vec![3u8];
        frame.extend_from_slice(&1u32.to_le_bytes());
        frame.extend_from_slice(&(data.len() as u32).to_le_bytes());
        frame.extend_from_slice(&data);
        for b in frame { invoke(0, PUT, i32::from(b)); }
        invoke(0, FLUSH, 0);
    }
}
