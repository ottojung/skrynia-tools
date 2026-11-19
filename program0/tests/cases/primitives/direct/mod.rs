mod caps;
mod collision;
mod lifecycle;
use super::{MemDisk, Rng};
use caps::NoList;
use skrynia_program0::wire::{Request, STATUS_ERR, STATUS_OK};
use skrynia_program0::{
    cap::{Disk, Eval},
    protocol::direct,
};
use skrynia_program0::{wdec::decode_res, wenc::encode_req};
struct Echo;
impl Eval for Echo {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        Ok([w, i].concat())
    }
}
fn call(disk: &NoList, request: Request) -> (u8, Vec<u8>) {
    let out = direct::handle(disk, &Rng(9), &Echo, &encode_req(&request));
    let res = decode_res(&out).unwrap();
    (res.status, res.body)
}
