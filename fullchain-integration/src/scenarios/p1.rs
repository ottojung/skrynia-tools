//! Raw P1 calls through the real trusted edge to the real kernel/P0/P1.
use webpush_sender::p1::{HttpP1, P1Transport, decode};

#[derive(Debug)]
pub struct P1Ok(pub Vec<u8>);
#[derive(Debug)]
pub struct P1Err(pub u8);

pub fn call(io: &HttpP1, req: &[u8]) -> Result<P1Ok, P1Err> {
    let raw = io.call(req).map_err(|_| P1Err(255))?;
    decode(&raw).map(P1Ok).map_err(|e| match e {
        webpush_sender::Error::P1(c) => P1Err(c),
        _ => P1Err(254),
    })
}
pub fn must(io: &HttpP1, req: &[u8], label: &str) -> Vec<u8> {
    let raw = io
        .call(req)
        .map_err(|e| panic!("{label}: io {e:?}"))
        .unwrap();
    match decode(&raw) {
        Ok(b) => b,
        Err(e) => panic!("{label}: decode {e:?} raw={:?}", &raw[..raw.len().min(16)]),
    }
}
