//! Disposable real-kernel fixture for the T10 shared userland lease.
//! Its only import is `p0.invoke`; roles come from the request payload.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}
const GET: i32 = 0;
const PUT: i32 = 1;
const FLUSH: i32 = 2;
const HELD: &str = "user/programs/appdata/t10lock/held.bin";
const RESULT: &str = "user/programs/appdata/t10lock/result.bin";

fn send(f: &[u8]) {
    unsafe {
        for &b in f {
            invoke(0, PUT, i32::from(b));
        }
        invoke(0, FLUSH, 0);
    }
}
fn reply() -> Vec<u8> {
    let mut out = Vec::new();
    unsafe {
        loop {
            let b = invoke(0, GET, 0);
            if b < 0 || b > 255 {
                break;
            }
            out.push(b as u8);
        }
    }
    out
}
fn code(r: &[u8]) -> i32 {
    if r.len() < 4 {
        return -99;
    }
    i32::from_le_bytes([r[0], r[1], r[2], r[3]])
}
fn word(n: u32) -> Vec<u8> {
    n.to_le_bytes().to_vec()
}
fn named(op: u8, path: &str, extra: &[u8]) -> i32 {
    let mut f = vec![op];
    f.extend(word(path.len() as u32));
    f.extend(path.as_bytes());
    f.extend(extra);
    send(&f);
    code(&reply())
}
fn acq() -> i32 {
    send(&[12]);
    code(&reply())
}
fn rel() -> i32 {
    send(&[13]);
    code(&reply())
}
fn write_file(path: &str, bytes: &[u8]) {
    let fd = named(1, path, &word(15));
    if fd > 1 {
        let mut w = vec![3u8];
        w.extend(word(fd as u32));
        w.extend(word(bytes.len() as u32));
        w.extend(bytes);
        send(&w);
        let mut s = vec![10u8];
        s.extend(word(fd as u32));
        send(&s);
        let mut c = vec![4u8];
        c.extend(word(fd as u32));
        send(&c);
    }
}
fn mark(path: &str) {
    write_file(path, b"1");
}
fn out(body: &[u8]) {
    let mut f = vec![3u8];
    f.extend(word(1));
    f.extend(word(body.len() as u32));
    f.extend(body);
    send(&f);
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let mut role = Vec::new();
    unsafe {
        loop {
            let b = invoke(0, GET, 0);
            if b < 0 || b > 255 {
                break;
            }
            role.push(b as u8);
        }
    }
    match role.first().copied().unwrap_or(b'?') {
        b'H' => {
            if acq() != 1 {
                out(b"LOCK_ACQUIRE_FAIL");
                return;
            }
            mark(HELD);
            // Kernel `exists` returns 0 for MISSING and 1 for present. Waiting
            // for > 0 keeps the lease held until the contender has published
            // its CAS verdict; treating 0 as present releases it immediately
            // and makes the exclusion check vacuous.
            for _ in 0..200000 {
                if named(14, RESULT, &[]) > 0 {
                    break;
                }
            }
            out(if rel() == 0 {
                b"LOCK_HELD_RELEASED"
            } else {
                b"LOCK_RELEASE_FAIL"
            });
        }
        b'C' => {
            let seen = acq();
            let rc = rel();
            let verdict = if seen == 0 {
                b"LOCK_CONTENDED".as_slice()
            } else if seen == 1 && rc == 0 {
                b"LOCK_ACQUIRED".as_slice()
            } else {
                b"LOCK_ERROR".as_slice()
            };
            // Publish the verdict before the holder is released, so the
            // contender's CAS is guaranteed to run inside the holder's lease.
            write_file(RESULT, verdict);
            out(verdict);
        }
        b'T' => {
            let _ = acq();
            mark(HELD);
            core::arch::wasm32::unreachable();
        }
        b'P' => {
            let seen = acq();
            let rc = rel();
            out(if seen == 1 && rc == 0 {
                b"LOCK_RECOVERED"
            } else if seen == 0 {
                b"LOCK_STUCK"
            } else {
                b"LOCK_ERROR"
            });
        }
        _ => out(b"LOCK_BAD_ROLE"),
    }
}
