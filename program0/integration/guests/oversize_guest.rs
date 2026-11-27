//! Disposable single-import child: oversized op15 frames fail closed.
//! Its only import is `p0.invoke`; no kernel import or linear-memory pointer
//! crosses the store boundary. Fake bearer and temp dirs only.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}

const CAP: usize = 2 * 1024 * 1024 - 1152;

fn put(frame: &[u8]) {
    unsafe {
        for &b in frame {
            invoke(0, 1, i32::from(b));
        }
        invoke(0, 2, 0);
    }
}

fn reply() -> Vec<u8> {
    let mut out = Vec::new();
    unsafe {
        loop {
            let b = invoke(0, 0, 0);
            if b < 0 {
                return out;
            }
            out.push(b as u8);
        }
    }
}

fn code(r: &[u8]) -> i32 {
    assert!(r.len() >= 4);
    i32::from_le_bytes([r[0], r[1], r[2], r[3]])
}

fn word(n: u32) -> Vec<u8> {
    n.to_le_bytes().to_vec()
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    reply();
    let mut huge = vec![15];
    huge.extend_from_slice(&u32::MAX.to_le_bytes());
    put(&huge);
    let mut ok = code(&reply()) == -1;

    let mut truncated = vec![15];
    truncated.extend_from_slice(&((CAP + 1) as u32).to_le_bytes());
    truncated.extend_from_slice(b"short");
    put(&truncated);
    ok &= code(&reply()) == -1;

    let body: &[u8] = if ok { b"T8_FRAMES_OK" } else { b"T8_FRAMES_FAIL" };
    let mut out = vec![3];
    out.extend_from_slice(&word(1));
    out.extend_from_slice(&word(body.len() as u32));
    out.extend_from_slice(body);
    put(&out);
}
