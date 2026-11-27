//! Verify the real P0 random-fill frame and response before filesystem mutations.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}
fn receive() -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let b = unsafe { invoke(0, 0, 0) };
        if b < 0 {
            return out;
        }
        out.push(b as u8);
    }
}
fn call(frame: &[u8]) -> Vec<u8> {
    for &b in frame {
        assert_eq!(unsafe { invoke(0, 1, i32::from(b)) }, 0);
    }
    assert_eq!(unsafe { invoke(0, 2, 0) }, 0);
    receive()
}
#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    receive();
    let first = call(&[11, 128, 0, 0, 0]);
    let second = call(&[11, 128, 0, 0, 0]);
    let ok = first.len() == 20
        && second.len() == 20
        && first[..4] == 16i32.to_le_bytes()
        && second[..4] == 16i32.to_le_bytes()
        && first[4..] != second[4..];
    let body: &[u8] = if ok { b"RANDOM_OK" } else { b"RANDOM_FAIL" };
    let mut frame = vec![3];
    frame.extend_from_slice(&1u32.to_le_bytes());
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(body);
    call(&frame);
}
