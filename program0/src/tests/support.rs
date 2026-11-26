//! Host-only helpers that drive the full PUT/FLUSH/GET relay path.
use super::{alloc, fake, invoke};

pub fn setup(files: Vec<(String, Vec<u8>)>) -> (i32, i32, i32) {
    let (i, o, r) = fake::seed(vec![], files);
    alloc::ensure(r).expect("arena");
    let (_slot, base) = alloc::acquire(r).expect("slot");
    invoke::enter(i, o, r, base, i, o, 0);
    (i, o, r)
}

pub fn call(frame: &[u8]) -> Vec<u8> {
    for &b in frame {
        assert_eq!(invoke::invoke(0, invoke::PUT, i32::from(b)), 0);
    }
    assert_eq!(invoke::invoke(0, invoke::FLUSH, 0), 0);
    let mut out = Vec::new();
    loop {
        let b = invoke::invoke(0, invoke::GET, 0);
        if b < 0 {
            break;
        }
        out.push(b as u8);
    }
    out
}

pub fn word(n: u32) -> Vec<u8> {
    n.to_le_bytes().to_vec()
}

pub fn path_frame(op: u8, path: &str) -> Vec<u8> {
    let mut f = vec![op];
    f.extend_from_slice(&word(path.len() as u32));
    f.extend_from_slice(path.as_bytes());
    f
}

pub fn code(reply: &[u8]) -> i32 {
    assert!(reply.len() >= 4);
    i32::from_le_bytes([reply[0], reply[1], reply[2], reply[3]])
}
