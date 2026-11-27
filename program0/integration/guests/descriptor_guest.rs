//! Disposable single-import child exercising framed descriptor ops 5-10.
//! Its only import is `p0.invoke`; every kernel operation is a framed byte
//! stream. No linear-memory pointer or function-table index crosses stores.
#[link(wasm_import_module = "p0")]
unsafe extern "C" {
    fn invoke(ctx: i32, op: i32, value: i32) -> i64;
}

const GET: i32 = 0;
const PUT: i32 = 1;
const FLUSH: i32 = 2;
// K10 routes kernel FS paths by their explicit root token; this fixture's
// writable data lives in the user root, never the operator staging root.
const DIR: &str = "user/programs/appdata/p0ops";
const NEW: &str = "user/programs/appdata/p0ops/probe.bin";
const RENAMED: &str = "user/programs/appdata/p0ops/renamed.bin";

fn sent(frame: &[u8]) {
    unsafe {
        for &b in frame {
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
            if b < 0 {
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
    f.extend_from_slice(&word(path.len() as u32));
    f.extend_from_slice(path.as_bytes());
    f.extend_from_slice(extra);
    sent(&f);
    code(&reply())
}

fn sync(fd: u32) -> i32 {
    let mut f = vec![10u8];
    f.extend_from_slice(&word(fd));
    sent(&f);
    code(&reply())
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    reply(); // drain the opaque request
    let mut ok = named(5, DIR, &word(4)) == 0;
    let mut f = vec![1u8];
    f.extend_from_slice(&word(NEW.len() as u32));
    f.extend_from_slice(NEW.as_bytes());
    f.extend_from_slice(&word(15));
    sent(&f);
    let fd = code(&reply());
    ok &= fd > 1;
    let mut w = vec![3u8];
    w.extend_from_slice(&word(fd as u32));
    w.extend_from_slice(&word(4));
    w.extend_from_slice(b"p0op");
    sent(&w);
    ok &= code(&reply()) == 4;
    ok &= sync(fd as u32) == 0;
    let mut rn = vec![7u8];
    rn.extend_from_slice(&word(NEW.len() as u32));
    rn.extend_from_slice(NEW.as_bytes());
    rn.extend_from_slice(&word(RENAMED.len() as u32));
    rn.extend_from_slice(RENAMED.as_bytes());
    sent(&rn);
    ok &= code(&reply()) == 0;
    ok &= named(6, RENAMED, &[]) == 0;
    let dfd = named(8, DIR, &[]);
    ok &= dfd > 1;
    let mut nx = vec![9u8];
    nx.extend_from_slice(&word(dfd as u32));
    nx.extend_from_slice(&word(256));
    sent(&nx);
    let entry = reply();
    ok &= code(&entry) > 0 && &entry[4..] == b"seed.bin";
    ok &= sync(dfd as u32) == 0;
    let body: &[u8] = if ok { b"OPS5TO10_OK" } else { b"OPS5TO10_FAIL" };
    let mut out = vec![3u8];
    out.extend_from_slice(&word(1));
    out.extend_from_slice(&word(body.len() as u32));
    out.extend_from_slice(body);
    sent(&out);
}
