//! Install a fresh deterministic machine for one host test.
use super::{Kind, S, State};

pub fn seed(input: Vec<u8>, files: Vec<(String, Vec<u8>)>) -> (i32, i32, i32) {
    S.with(|s| {
        let mut b = s.borrow_mut();
        *b = State::new();
        b.input = input;
        b.files = files;
        let i = b.fd(Kind::Input);
        let o = b.fd(Kind::Output);
        let r = b.fd(Kind::Ram);
        (i, o, r)
    })
}

/// Replace only the inbound request, keeping RAM/CAS state across requests.
pub fn request(bytes: Vec<u8>) -> (i32, i32) {
    S.with(|s| {
        let mut b = s.borrow_mut();
        b.input = bytes;
        b.inp = 0;
        (b.fd(Kind::Input), b.fd(Kind::Output))
    })
}
