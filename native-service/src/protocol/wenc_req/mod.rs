mod encode;
mod encode_admin;
mod encode_app;
mod encode_push;
mod encode_sender;
mod helpers;
use crate::wire::{Request, VERSION};

pub fn encode_req(req: &Request) -> Vec<u8> {
    let mut out = vec![VERSION];
    encode::encode_body(&mut out, req);
    out
}
