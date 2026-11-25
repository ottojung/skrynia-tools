//! OP27 helper methods on the fixture.
use super::disk::{Eval0, Rng};
use super::support::Fixture;
use skrynia_native_service::{
    NoLock,
    handle::handle,
    hash_bytes,
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, Response},
};

impl Fixture {
    pub fn call(&self, req: Request) -> Response {
        decode_res(&handle(
            &self.reg,
            &self.disk,
            &NoLock,
            &Rng,
            &Eval0,
            &encode_req(&req),
        ))
        .unwrap()
    }
    pub fn expected(&self) -> Vec<u8> {
        hash_bytes(&self.subscription).to_vec()
    }
    pub fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        self.disk
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Fixture {
    pub fn status(&self, key: &[u8], sub: &[u8], cap: &[u8], hash: Vec<u8>) -> (u8, Vec<u8>) {
        let out = self.call(Request::SubStatus(
            cap.to_vec(),
            key.to_vec(),
            sub.to_vec(),
            hash,
        ));
        (out.status, out.body)
    }
}
